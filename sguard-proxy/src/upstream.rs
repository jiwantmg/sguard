use bytes::Bytes;
use http_body_util::{combinators::UnsyncBoxBody, BodyExt, Empty};
use hyper::{
    body::{Body, Incoming},
    header::{HeaderName, HeaderValue, HOST},
    Method, Request, Response, Uri,
};
use hyper_util::{
    client::legacy::{connect::HttpConnector, Client, Error as ClientError},
    rt::{TokioExecutor, TokioTimer},
};
use sguard_core::model::context::RequestContext;
use sguard_error::{Error as AppError, ErrorType};
use std::error::Error as StdError;
use std::io::ErrorKind;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

type UpBody = UnsyncBoxBody<Bytes, hyper::Error>;

/// Pool tuning. `POOL_IDLE_TIMEOUT` must stay BELOW the upstream's keep-alive timeout
/// (nginx default 75s), otherwise the pool hands out sockets the upstream already closed.
const POOL_IDLE_TIMEOUT: Duration = Duration::from_secs(20);
/// Must be >= MAX_INFLIGHT_UPSTREAM in state_machine.rs, or the pool closes and
/// reopens connections under load (connection churn).
const POOL_MAX_IDLE_PER_HOST: usize = 2048;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(3);
const RESPONSE_TIMEOUT: Duration = Duration::from_secs(30);

/// Headers that apply to a single connection and must not be forwarded.
const HOP_BY_HOP: [&str; 8] = [
    "connection",
    "keep-alive",
    "proxy-authenticate",
    "proxy-authorization",
    "te",
    "trailer",
    "transfer-encoding",
    "upgrade",
];

/// Why an upstream call failed. Each kind has a different root cause and a different fix,
/// so they are counted separately instead of all being reported as "ConnectTimeout".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureKind {
    ConnectRefused,    // upstream down or not listening
    ConnectTimeout,    // SYN dropped: accept queue full / firewall / overload
    ConnectFailed,     // other connect error
    ResourceExhausted, // gateway ran out of fds or ephemeral ports (EMFILE, EADDRNOTAVAIL...)
    StaleConnection,   // reused pooled socket was already closed by the upstream
    ResponseTimeout,   // upstream accepted the request but did not answer in time
    Other,
}

impl FailureKind {
    pub const ALL: [FailureKind; 7] = [
        FailureKind::ConnectRefused,
        FailureKind::ConnectTimeout,
        FailureKind::ConnectFailed,
        FailureKind::ResourceExhausted,
        FailureKind::StaleConnection,
        FailureKind::ResponseTimeout,
        FailureKind::Other,
    ];
}

#[derive(Default)]
pub struct UpstreamStats {
    failures: [AtomicU64; 7],
    pub retries: AtomicU64,
}

impl UpstreamStats {
    fn record(&self, kind: FailureKind) {
        self.failures[kind as usize].fetch_add(1, Ordering::Relaxed);
    }
    pub fn snapshot(&self) -> [u64; 7] {
        let mut out = [0u64; 7];
        for (i, c) in self.failures.iter().enumerate() {
            out[i] = c.load(Ordering::Relaxed);
        }
        out
    }
}

pub struct UpstreamService {
    // Cheap to clone; all clones share ONE connection pool.
    client: Client<HttpConnector, UpBody>,
    pub stats: UpstreamStats,
}

impl UpstreamService {
    /// Build once at startup. Do NOT call per request.
    pub fn new() -> Self {
        let mut http = HttpConnector::new();
        http.set_nodelay(true);
        http.set_connect_timeout(Some(CONNECT_TIMEOUT));
        http.set_keepalive(Some(Duration::from_secs(30)));

        let client = Client::builder(TokioExecutor::new())
            .pool_timer(TokioTimer::new()) // required, or idle sockets are never evicted
            .pool_idle_timeout(POOL_IDLE_TIMEOUT)
            .pool_max_idle_per_host(POOL_MAX_IDLE_PER_HOST)
            .build(http);

        Self {
            client,
            stats: UpstreamStats::default(),
        }
    }

    pub async fn call_upstream_service(
        &self,
        req: &mut RequestContext,
    ) -> Result<Response<Incoming>, AppError> {
        // 1. Get the route's upstream base URI (use pre-parsed if available)
        let authority;
        let scheme;
        if let Some(parsed) = &req.route_definition.parsed_uri {
            authority = parsed.authority().cloned().ok_or_else(|| {
                AppError::new(ErrorType::Custom(String::from("Upstream uri has no host")))
            })?;
            scheme = parsed
                .scheme()
                .cloned()
                .unwrap_or(hyper::http::uri::Scheme::HTTP);
        } else {
            let base: Uri = req
                .route_definition
                .uri
                .parse()
                .map_err(|_| AppError::new(ErrorType::Custom(String::from("Invalid url"))))?;
            authority = base.authority().cloned().ok_or_else(|| {
                AppError::new(ErrorType::Custom(String::from("Upstream uri has no host")))
            })?;
            scheme = base
                .scheme()
                .cloned()
                .unwrap_or(hyper::http::uri::Scheme::HTTP);
        }

        // 2. Take the client's request apart (error instead of panic)
        let body = req.request.incoming.take().ok_or_else(|| {
            AppError::new(ErrorType::Custom(String::from("Request body already consumed")))
        })?;
        let mut parts = req.request.take_parts();

        // 3. Upstream scheme + authority, client's path and query
        let path_and_query = parts
            .uri
            .path_and_query()
            .cloned()
            .unwrap_or_else(|| hyper::http::uri::PathAndQuery::from_static("/"));
        parts.uri = Uri::builder()
            .scheme(scheme)
            .authority(authority.clone())
            .path_and_query(path_and_query)
            .build()
            .map_err(|_| {
                AppError::new(ErrorType::Custom(String::from("Could not build upstream uri")))
            })?;

        // 4. Strip hop-by-hop headers and set Host
        strip_hop_by_hop(&mut parts.headers);
        if let Ok(host) = HeaderValue::from_str(authority.as_str()) {
            parts.headers.insert(HOST, host);
        }

        // 5. Only bodyless idempotent requests may be replayed after a stale-socket failure.
        let replay = (matches!(parts.method, Method::GET | Method::HEAD | Method::OPTIONS)
            && body.is_end_stream())
        .then(|| {
            (
                parts.method.clone(),
                parts.uri.clone(),
                parts.version,
                parts.headers.clone(),
            )
        });

        let first = Request::from_parts(parts, body.boxed_unsync());
        match self.send(first).await {
            Ok(resp) => Ok(resp),
            Err(FailureKind::StaleConnection) if replay.is_some() => {
                let (method, uri, version, headers) = replay.unwrap();
                self.stats.retries.fetch_add(1, Ordering::Relaxed);
                let empty: UpBody = Empty::<Bytes>::new()
                    .map_err(|never| -> hyper::Error { match never {} })
                    .boxed_unsync();
                let mut retry = Request::new(empty);
                *retry.method_mut() = method;
                *retry.uri_mut() = uri;
                *retry.version_mut() = version;
                *retry.headers_mut() = headers;
                self.send(retry).await.map_err(to_app_error)
            }
            Err(kind) => Err(to_app_error(kind)),
        }
    }

    async fn send(&self, req: Request<UpBody>) -> Result<Response<Incoming>, FailureKind> {
        match tokio::time::timeout(RESPONSE_TIMEOUT, self.client.request(req)).await {
            Err(_) => {
                self.stats.record(FailureKind::ResponseTimeout);
                Err(FailureKind::ResponseTimeout)
            }
            Ok(Err(err)) => {
                let kind = classify(&err);
                self.stats.record(kind);
                // debug, not warn: the periodic summary reports counts, so a failure
                // storm cannot also become a log storm
                log::debug!("upstream failure kind={:?} err={} source={:?}", kind, err, err.source());
                Err(kind)
            }
            Ok(Ok(resp)) => Ok(resp),
        }
    }
}

impl Default for UpstreamService {
    fn default() -> Self {
        Self::new()
    }
}

fn to_app_error(kind: FailureKind) -> AppError {
    match kind {
        FailureKind::ConnectTimeout | FailureKind::ResponseTimeout => {
            AppError::new(ErrorType::ConnectTimeout) // -> 504
        }
        FailureKind::ResourceExhausted => {
            AppError::new(ErrorType::Custom(String::from("Upstream overloaded"))) // -> 503
        }
        other => AppError::new(ErrorType::Custom(format!("Upstream error: {:?}", other))), // -> 502
    }
}

/// Walk the whole error chain and decide what actually went wrong.
fn classify(err: &ClientError) -> FailureKind {
    let is_connect = err.is_connect();
    let mut cur: Option<&(dyn StdError + 'static)> = Some(err);
    while let Some(e) = cur {
        if let Some(io) = e.downcast_ref::<std::io::Error>() {
            // 24 EMFILE, 23 ENFILE, 99 EADDRNOTAVAIL, 105 ENOBUFS
            if matches!(io.raw_os_error(), Some(24) | Some(23) | Some(99) | Some(105)) {
                return FailureKind::ResourceExhausted;
            }
            match io.kind() {
                ErrorKind::ConnectionRefused => return FailureKind::ConnectRefused,
                ErrorKind::TimedOut => return FailureKind::ConnectTimeout,
                ErrorKind::BrokenPipe
                | ErrorKind::ConnectionReset
                | ErrorKind::ConnectionAborted
                | ErrorKind::UnexpectedEof
                | ErrorKind::NotConnected
                    if !is_connect =>
                {
                    return FailureKind::StaleConnection
                }
                _ => {}
            }
        }
        if let Some(h) = e.downcast_ref::<hyper::Error>() {
            if h.is_canceled() || h.is_closed() || h.is_incomplete_message() {
                return FailureKind::StaleConnection;
            }
        }
        cur = e.source();
    }
    if is_connect {
        FailureKind::ConnectFailed
    } else {
        FailureKind::Other
    }
}

fn strip_hop_by_hop(headers: &mut hyper::HeaderMap) {
    for name in HOP_BY_HOP {
        headers.remove(name);
    }
    // Headers named in `Connection: a, b` are hop-by-hop too
    if let Some(conn_val) = headers.get(hyper::header::CONNECTION) {
        if let Ok(s) = conn_val.to_str() {
            let extra: Vec<HeaderName> = s
                .split(',')
                .filter_map(|n| HeaderName::from_bytes(n.trim().as_bytes()).ok())
                .collect();
            for name in extra {
                headers.remove(name);
            }
        }
    }
}