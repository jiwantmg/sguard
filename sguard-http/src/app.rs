use hyper::service::{make_service_fn, service_fn};
use hyper::Server;
use sguard_error::Error;
use sguard_filter::auth::basic::SGuardBasicAuthFilter;
use sguard_filter::auth::AuthFilter;
use sguard_filter::core::Filter;
use sguard_filter::exception::ExceptionTranslationFilter;
use sguard_filter::filter_chain::FilterChain;
use sguard_filter::http::HeaderWriterFilter;
use sguard_filter::logging::LoggingFilter;
use sguard_filter::routing::RoutingFilter;
use sguard_filter::security::CsrfFilter;
use sguard_filter::session::SessionManagementFilter;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::upstream::UpstreamService;

pub struct AppBuilder {
    filter_chain: Arc<Mutex<FilterChain>>,
}

impl AppBuilder {
    pub fn new() -> Self {
        Self {
            filter_chain: Arc::new(Mutex::new(FilterChain::new(vec![]))),
        }
    }
    pub fn app_builder(&mut self) {
        let csrf_filter = Arc::new(CsrfFilter::new(None));
        let auth_filter = Arc::new(AuthFilter::new(Some(Arc::new(SGuardBasicAuthFilter))));

        let logging_filter = Arc::new(LoggingFilter::new(None));
        let session_management_filter = Arc::new(SessionManagementFilter::new(None));
        let exception_translation_filter = Arc::new(ExceptionTranslationFilter::new(None));
        let header_writer_filter = Arc::new(HeaderWriterFilter::new(None));
        let routing_filter = Arc::new(RoutingFilter::new());
        self.filter_chain = Arc::new(Mutex::new(FilterChain::new(vec![
            csrf_filter,
            auth_filter,
            logging_filter,
            session_management_filter,
            exception_translation_filter,
            header_writer_filter,
            routing_filter,
        ])));
    }

    pub async fn run(&self) {
        let make_svc = make_service_fn(move |_conn| {
            let filter_chain = self.filter_chain.clone();
            async move {
                Ok::<_, Error>(service_fn(move |req| {
                    let filter_chain = filter_chain.clone();
                    async move {
                        let state_machine_service = UpstreamService::new();
                        let state_machine_handler = state_machine_service.create_handler();
                        let filter_chain = filter_chain.lock().await;
                        let result = filter_chain.handle(&req, state_machine_handler).await;
                        match result {
                            Ok(response) => Ok(response),
                            Err(e) => Result::Err(e),
                        }
                    }
                }))
            }
        });
        let addr = ([127, 0, 0, 1], 8080).into();
        let server = Server::bind(&addr).serve(make_svc);

        println!("Listening on http://{}", addr);

        if let Err(e) = server.await {
            eprintln!("Server error: {}", e);
        }
    }
}
