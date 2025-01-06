use hyper::service::{make_service_fn, service_fn};
use hyper::Server;
use sguard_core::filter::Filter;
use sguard_core::model::context::RequestContext;
use sguard_core::model::route::RouteDefinition;
use sguard_error::Error;
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

use crate::upstream::UpstreamService;

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

    pub async fn run(&self) {
        let state_machine_handler = self.upstream_service.create_handler();
        let make_svc = make_service_fn(move |_conn| {
            let filter_chain = self.filter_chain.clone();
            let state_machine_handler = state_machine_handler.clone();
            async move {
                Ok::<_, Error>(service_fn(move |req| {
                    let filter_chain = filter_chain.clone();
                    let state_machine_handler = state_machine_handler.clone();
                    async move {
                        let filter_chain = filter_chain.read().await;
                        let result = filter_chain.handle(&mut RequestContext{
                                                        request: req,
                                                        route_definition: RouteDefinition{
                                                            id: String::from(""),
                                                            filters: vec![],
                                                            predicates: vec![]
                                                        }
                                                    }, 
                                                state_machine_handler
                                                ).await;
                        match result {
                            Ok(response) => Ok(response),
                            Err(e) => Result::Err(e),
                        }
                    }
                }))
            }
        });
        let addr = ([0, 0, 0, 0], 8080).into();
        let server = Server::bind(&addr).serve(make_svc);

        println!("Listening on http://{}", addr);

        if let Err(e) = server.await {
            eprintln!("Server error: {}", e);
        }
    }
}
