use crate::upstream::UpstreamService;
use bytes::Bytes;
use http::{
    header::{CONTENT_TYPE, RETRY_AFTER},
    Response, StatusCode,
};
use http_body_util::{Either, Full};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::{TokioIo, TokioTimer};
use sguard_core::filter::{Filter, FilterFn, ResponseBody};
use sguard_core::model::context::RequestContext;
use sguard_core::model::core::HttpRequest;
use sguard_core::model::route::RouteDefinition;
use sguard_error::{Error as SguardError, ErrorType};
use sguard_filter::auth::basic::SGuardBasicAuthFilter;
use sguard_filter::auth::AuthFilter;
use sguard_filter::exception::ExceptionTranslationFilter;
use sguard_filter::filter_chain::FilterChain;
use sguard_filter::http::HeaderWriterFilter;
use sguard_filter::logging::LoggingFilter;
use sguard_filter::security::CsrfFilter;
use sguard_filter::session::SessionManagementFilter;
use sguard_proxy::state_machine::StateMachineManager;
use sguard_routing::filter::RoutingFilter;
use std::convert::Infallible;
use std::error::Error as StdError;
use std::io::ErrorKind;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpSocket;
use tokio::sync::Semaphore;

/// Kernel accept-queue size for the listener (also capped by net.core.somaxconn).
const LISTEN_BACKLOG: u32 = 4096;

fn detect_max_client_connections() -> usize {
    #[cfg(unix)]
    unsafe {
        let mut rlim = libc::rlimit {
            rlim_cur: 0,
            rlim_max: 0,
        };
        if libc::getrlimit(libc::RLIMIT_NOFILE, &mut rlim) == 0 && rlim.rlim_cur > 100 {
            let safe_limit = ((rlim.rlim_cur - 100) / 2) as usize;
            let capped = safe_limit.min(65_536);
            log::info!(
                "Configured max client connections to {} based on OS rlimit ({})",
                capped,
                rlim.rlim_cur
            );
            return capped;
        }
    }
    10_000
}

/// ASSUMPTION 1: `ResponseBody` implements `http_body::Body<Data = Bytes>` with an error type
/// that converts into `Box<dyn Error + Send + Sync>`. It must already, because hyper accepts
/// `Response<ResponseBody>` today. `Either` lets one type carry the proxied body or a
/// locally built error body.
type GatewayBody = Either<ResponseBody, Full<Bytes>>;

/// Turns any filter or upstream error into a real HTTP response, so hyper never has to
/// drop the connection because a filter returned `Err`.
fn error_response(err: &SguardError) -> Response<GatewayBody> {
    // ASSUMPTION 2: only ConnectTimeout, ReadError and Custom(String) are known to me.
    // Auth/CSRF errors are matched by name below as a stopgap. Replace that with real
    // `ErrorType::...` arms once you know your auth error variants.
    let name = format!("{:?}", err.etype);

    let status = match &err.etype {
        ErrorType::ConnectTimeout => StatusCode::GATEWAY_TIMEOUT, // 504
        ErrorType::ReadError => StatusCode::BAD_GATEWAY,          // 502
        ErrorType::Custom(msg) if msg == "Bad request" => StatusCode::BAD_REQUEST,
        ErrorType::Custom(msg) if msg == "Upstream overloaded" => StatusCode::SERVICE_UNAVAILABLE,
        ErrorType::Custom(msg) if msg == "Bad gateway" || msg.starts_with("Upstream error") => {
            StatusCode::BAD_GATEWAY
        }
        _ if name.contains("InvalidCookie")
            || name.contains("Unauthorized")
            || name.contains("Authentication") =>
        {
            StatusCode::UNAUTHORIZED
        }
        _ if name.contains("Forbidden") || name.contains("Csrf") => StatusCode::FORBIDDEN,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    };

    if status.is_server_error() {
        log::debug!("Responding {} for error: {:?}", status, err);
    } else {
        log::trace!("Responding {} for error: {:?}", status, err);
    }

    let text = status.canonical_reason().unwrap_or("error");
    let mut builder = Response::builder()
        .status(status)
        .header(CONTENT_TYPE, "text/plain; charset=utf-8");
    if status == StatusCode::SERVICE_UNAVAILABLE {
        builder = builder.header(RETRY_AFTER, "1");
    }
    builder
        .body(Either::Right(Full::new(Bytes::from(text))))
        .expect("static response parts are valid")
}

/// True when the error only means the client went away.
fn is_client_disconnect(err: &(dyn StdError + 'static)) -> bool {
    let mut cur: Option<&(dyn StdError + 'static)> = Some(err);
    while let Some(e) = cur {
        if let Some(io) = e.downcast_ref::<std::io::Error>() {
            if matches!(
                io.kind(),
                ErrorKind::NotConnected
                    | ErrorKind::BrokenPipe
                    | ErrorKind::ConnectionReset
                    | ErrorKind::ConnectionAborted
                    | ErrorKind::UnexpectedEof
            ) {
                return true;
            }
        }
        cur = e.source();
    }
    false
}

pub struct AppBuilder {
    filter_chain: Arc<FilterChain>,
    upstream_service: UpstreamService,
}

impl AppBuilder {
    pub fn new() -> Self {
        Self {
            filter_chain: Arc::new(FilterChain::new(vec![])),
            upstream_service: UpstreamService::new(Arc::new(StateMachineManager::new())),
        }
    }

    pub fn app_builder(&mut self) {
        let csrf_filter = Arc::new(CsrfFilter::new(None));
        let auth_filter = Arc::new(AuthFilter::new(Some(Arc::new(SGuardBasicAuthFilter))));

        let logging_filter = Arc::new(LoggingFilter::new(None));
        let session_management_filter = Arc::new(SessionManagementFilter::new(None));
        let exception_translation_filter = Arc::new(ExceptionTranslationFilter::new(None));
        let header_writer_filter = Arc::new(HeaderWriterFilter::new(None));

        let mut routing_filter = RoutingFilter::new("routes.yaml");
        routing_filter.configure_routes();
        let routing_filter = Arc::new(routing_filter);

        self.filter_chain = Arc::new(FilterChain::new(vec![
            csrf_filter,
            auth_filter,
            logging_filter,
            session_management_filter,
            exception_translation_filter,
            header_writer_filter,
            routing_filter,
        ]));

        log::debug!("Creating upstream service");
    }

    pub async fn run(&self) -> Result<Infallible, std::io::Error> {
        // Compose the filter chain around the upstream handler ONCE, not per request
        let terminal = self.upstream_service.create_handler();
        let chain: FilterFn = self.filter_chain.build(terminal);

        let svc = service_fn(move |req: http::Request<hyper::body::Incoming>| {
            let chain = chain.clone(); // one Arc clone per request
            async move {
                let result = match HttpRequest::from_hyper_request(req).await {
                    Ok(http_request) => {
                        let mut ctx = RequestContext {
                            request: http_request,
                            route_definition: RouteDefinition::default(),
                        };
                        chain(&mut ctx).await
                    }
                    Err(err) => {
                        log::debug!("Could not parse request: {:?}", err);
                        Err(SguardError::new(ErrorType::Custom(String::from("Bad request"))))
                    }
                };

                // Infallible: every outcome, including every error, is a response
                Ok::<Response<GatewayBody>, Infallible>(match result {
                    Ok(response) => response.map(Either::Left),
                    Err(err) => error_response(&err),
                })
            }
        });

        let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
        let socket = TcpSocket::new_v4()?;
        socket.set_reuseaddr(true)?;
        #[cfg(target_os = "linux")]
        let _ = socket.set_reuseport(true);
        socket.bind(addr)?;
        let listener = socket.listen(LISTEN_BACKLOG)?;
        log::info!("HTTP server is running on http://{}", addr);

        let conn_limit = Arc::new(Semaphore::new(detect_max_client_connections()));

        loop {
            // At the cap we stop accepting; extra clients wait in the kernel backlog
            // instead of exhausting file descriptors.
            let permit = conn_limit
                .clone()
                .acquire_owned()
                .await
                .expect("connection semaphore is never closed");

            let (stream, _) = match listener.accept().await {
                Ok(conn) => conn,
                Err(err) => {
                    // EMFILE and similar are temporary; don't take the gateway down
                    log::error!("accept failed: {}", err);
                    tokio::time::sleep(Duration::from_millis(50)).await;
                    continue;
                }
            };
            let _ = stream.set_nodelay(true);
            let io = TokioIo::new(stream);
            let service = svc.clone();

            tokio::spawn(async move {
                let _permit = permit; // released when the connection ends

                let mut builder = http1::Builder::new();
                builder
                    .timer(TokioTimer::new()) // required for header_read_timeout
                    .header_read_timeout(Duration::from_secs(10))
                    .keep_alive(true);

                if let Err(err) = builder.serve_connection(io, service).await {
                    if is_client_disconnect(&err)
                        || err.is_incomplete_message()
                        || err.is_canceled()
                    {
                        log::debug!("client disconnected: {}", err);
                    } else {
                        log::warn!("connection error: {}", err);
                    }
                }
            });
        }
    }
}