use crate::upstream::UpstreamService;
use http::{Response, StatusCode};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::{TokioIo, TokioTimer};
use sguard_core::filter::{Filter, ResponseBody};
use sguard_core::model::context::RequestContext;
use sguard_core::model::core::HttpRequest;
use sguard_core::model::route::RouteDefinition;
use sguard_error::{Error as SguardError, ErrorType, IntoHyperResponse};
use sguard_filter::auth::basic::SGuardBasicAuthFilter;
use sguard_filter::auth::AuthFilter;
use sguard_filter::exception::ExceptionTranslationFilter;
use sguard_filter::filter_chain::FilterChain;
use sguard_filter::http::HeaderWriterFilter;
use sguard_filter::logging::LoggingFilter;
use sguard_filter::routing::BaseRoutingFilter;
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
use tokio::net::TcpListener;
use tokio::sync::RwLock;

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
    filter_chain: Arc<RwLock<FilterChain>>,
    upstream_service: UpstreamService,
}

impl AppBuilder {
    pub fn new() -> Self {
        Self {
            filter_chain: Arc::new(RwLock::new(FilterChain::new(vec![]))),
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
        let routing_filter_mut = Arc::from(routing_filter);
        let routing_filter = Arc::new(BaseRoutingFilter::new(Some(routing_filter_mut)));

        self.filter_chain = Arc::new(RwLock::new(FilterChain::new(vec![
            csrf_filter,
            auth_filter,
            logging_filter,
            session_management_filter,
            exception_translation_filter,
            header_writer_filter,
            routing_filter,
        ])));

        log::debug!("Creating upstream service");
    }

    // app_builder(): unchanged from your original

    pub async fn run(&self) -> Result<Infallible, std::io::Error> {
        let state_machine_handler = self.upstream_service.create_handler();
        let filter_chain = self.filter_chain.clone();

        let svc = service_fn(move |req: http::Request<hyper::body::Incoming>| {
            let filter_chain = filter_chain.clone();
            let state_machine_handler = state_machine_handler.clone();
            async move {
                let http_request = match HttpRequest::from_hyper_request(req).await {
                    Ok(r) => r,
                    Err(err) => {
                        log::debug!("Could not parse request: {:?}", err);
                        return Err(SguardError::new(ErrorType::Custom(String::from("Bad request"))));
                    }
                };

                let filter_chain = filter_chain.read().await;
                filter_chain
                    .handle(
                        &mut RequestContext {
                            request: http_request,
                            route_definition: RouteDefinition::default(),
                        },
                        state_machine_handler,
                    )
                    .await
            }
        });

        let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
        let listener = TcpListener::bind(&addr).await?;
        log::info!("HTTP server is running on http://{}", addr);

        loop {
            let (stream, _) = match listener.accept().await {
                Ok(conn) => conn,
                Err(err) => {
                    log::error!("accept failed: {}", err);
                    tokio::time::sleep(Duration::from_millis(50)).await;
                    continue;
                }
            };
            let _ = stream.set_nodelay(true);
            let io = TokioIo::new(stream);
            let service = svc.clone();

            tokio::spawn(async move {
                let mut builder = http1::Builder::new();
                builder
                    .timer(TokioTimer::new())
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