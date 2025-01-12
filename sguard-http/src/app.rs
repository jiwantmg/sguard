use hyper::service::service_fn;
use sguard_core::model::core::HttpRequest;
use std::convert::Infallible;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use sguard_core::model::context::RequestContext;
use sguard_core::model::route::RouteDefinition;
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
use std::sync::Arc;
use tokio::sync::RwLock;
use hyper_util::rt::TokioIo;
use crate::upstream::UpstreamService;
use sguard_core::filter::Filter;
use hyper::server::conn::http1;

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

        let mut routing_filter =RoutingFilter::new("routes.yaml");
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

    
    pub async fn run(&self) -> Result<Infallible, std::io::Error> {
        let state_machine_handler = self.upstream_service.create_handler();
        // let make_svc = make_service_fn(move |_conn| {
        let filter_chain = self.filter_chain.clone();
        //     let state_machine_handler = state_machine_handler.clone();
        let svc = service_fn(move |req: http::Request<hyper::body::Incoming>| {
            let filter_chain = filter_chain.clone();
            let state_machine_handler = state_machine_handler.clone();
            async move {
                let filter_chain = filter_chain.read().await;                
                let http_request = HttpRequest::from_hyper_request(req).await;
                if http_request.is_err() {
                    eprintln!("Error processing request");
                }                
                let result = filter_chain.handle(&mut RequestContext{
                                                request: http_request.unwrap(),
                                                route_definition: RouteDefinition::default()
                                            }, 
                                        state_machine_handler
                                        ).await;
                match result {
                    Ok(response) => Result::Ok(response),
                    Err(e) => Result::Err(e),
                }
            }
        });

        let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
        let listener = TcpListener::bind(&addr).await?;
        println!("HTTP server is running on http://0.0.0.0:8080");
        loop {
            let (stream, _) = listener.accept().await?;
            let io = TokioIo::new(stream);
            let service = svc.clone();
            tokio::spawn(async move {
                // N.B. should use hyper service_fn here, since it's required to be implemented hyper Service trait!
                let builder: http1::Builder = http1::Builder::new();
                if let Err(err) = builder.serve_connection(io, service).await {
                    eprintln!("server error: {}", err);
                }
            });
        }
    }
}
