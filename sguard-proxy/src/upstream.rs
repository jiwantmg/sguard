use hyper::{
    body::Incoming,
    header::{HeaderName, HeaderValue, HOST},
    Request, Response, Uri,
};
use hyper_util::{
    client::legacy::{connect::HttpConnector, Client},
    rt::TokioExecutor,
};
use sguard_core::model::context::RequestContext;
use sguard_error::{Error as AppError, ErrorType};
use std::time::Duration;

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

#[derive(Clone)]
pub struct UpstreamService {
    // Client is cheap to clone; all clones share ONE connection pool.
    client: Client<HttpConnector, Incoming>,
    request_timeout: Duration,
}

impl UpstreamService {
    /// Build once at startup and share (clone or Arc). Do NOT call per request.
    pub fn new() -> Self {
        let mut http = HttpConnector::new();
        http.set_nodelay(true);
        http.set_connect_timeout(Some(Duration::from_secs(3)));
        http.set_keepalive(Some(Duration::from_secs(30)));

        let client = Client::builder(TokioExecutor::new())
            .pool_idle_timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(512)
            .build(http);

        Self {
            client,
            request_timeout: Duration::from_secs(30),
        }
    }

    pub async fn call_upstream_service(
        &self,
        req: &mut RequestContext,
    ) -> Result<Response<Incoming>, AppError> {
        // 1. Parse the route's upstream base URI
        let base: Uri = req
            .route_definition
            .uri
            .parse()
            .map_err(|_| AppError::new(ErrorType::Custom(String::from("Invalid url"))))?;

        let authority = base
            .authority()
            .cloned()
            .ok_or_else(|| AppError::new(ErrorType::Custom(String::from("Upstream uri has no host"))))?;
        let scheme = base.scheme().cloned().unwrap_or(hyper::http::uri::Scheme::HTTP);

        // 2. Take the client's request apart (no unwrap: return an error instead of panicking)
        let body = req
            .request
            .incoming
            .take()
            .ok_or_else(|| AppError::new(ErrorType::Custom(String::from("Request body already consumed"))))?;
        let mut parts = req
            .request
            .parts
            .take()
            .ok_or_else(|| AppError::new(ErrorType::Custom(String::from("Request parts already consumed"))))?;

        // 3. Rewrite the target: upstream scheme + authority, client's path and query
        let path_and_query = parts
            .uri
            .path_and_query()
            .cloned()
            .unwrap_or_else(|| hyper::http::uri::PathAndQuery::from_static("/"));

        // NOTE: if the route has a base path (e.g. http://host:9000/api) and you want it
        // prefixed or the route prefix stripped, do it here.
        parts.uri = Uri::builder()
            .scheme(scheme)
            .authority(authority.clone())
            .path_and_query(path_and_query)
            .build()
            .map_err(|_| AppError::new(ErrorType::Custom(String::from("Could not build upstream uri"))))?;

        // 4. Strip hop-by-hop headers (also any named in the Connection header) and fix Host
        strip_hop_by_hop(&mut parts.headers);
        if let Ok(host) = HeaderValue::from_str(authority.as_str()) {
            parts.headers.insert(HOST, host);
        }

        let upstream_req = Request::from_parts(parts, body);

        // 5. Send through the pooled client, with an overall deadline
        match tokio::time::timeout(self.request_timeout, self.client.request(upstream_req)).await {
            Err(_) => {
                log::warn!("Upstream request timed out");
                Err(AppError::new(ErrorType::ConnectTimeout))
            }
            Ok(Err(err)) => {
                log::warn!("Upstream request failed: {}", err);
                if err.is_connect() {
                    Err(AppError::new(ErrorType::ConnectTimeout)) // see note below
                } else {
                    Err(AppError::new(ErrorType::Custom(format!("Upstream error: {}", err))))
                }
            }
            // The returned body is `Incoming`, same type as before, so callers are unchanged.
            Ok(Ok(resp)) => Ok(resp.map(|b| b)),
        }
    }
}

impl Default for UpstreamService {
    fn default() -> Self {
        Self::new()
    }
}

fn strip_hop_by_hop(headers: &mut hyper::HeaderMap) {
    // Headers listed in `Connection: a, b` are hop-by-hop too
    let extra: Vec<HeaderName> = headers
        .get_all(hyper::header::CONNECTION)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|v| v.split(','))
        .filter_map(|n| HeaderName::from_bytes(n.trim().as_bytes()).ok())
        .collect();

    for name in HOP_BY_HOP {
        headers.remove(name);
    }
    for name in extra {
        headers.remove(name);
    }
}