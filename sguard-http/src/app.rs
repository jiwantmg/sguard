use hyper::service::{make_service_fn, service_fn};
use hyper::Server;
use serde;
use serde::{Deserialize, Serialize};
use sguard_core::http::ResponseEntity;
use sguard_error::Error;
use sguard_filter::auth::basic::SGuardBasicAuthFilter;
use sguard_filter::auth::AuthFilter;
use sguard_filter::core::{Filter, FilterFn};
use sguard_filter::exception::ExceptionTranslationFilter;
use sguard_filter::filter_chain::FilterChain;
use sguard_filter::http::HeaderWriterFilter;
use sguard_filter::logging::LoggingFilter;
use sguard_filter::routing::RoutingFilter;
use sguard_filter::security::CsrfFilter;
use sguard_filter::session::SessionManagementFilter;
use sguard_filter::upstream::UpstreamtFilter;
use std::sync::Arc;
use tokio::sync::Mutex;

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
        //let logout_filter = Arc::new(LogoutFilter::new(optional_empty_filter_chain.clone()));
        let session_management_filter = Arc::new(SessionManagementFilter::new(None));
        let exception_translation_filter = Arc::new(ExceptionTranslationFilter::new(None));
        let header_writer_filter = Arc::new(HeaderWriterFilter::new(None));
        let routing_filter = Arc::new(RoutingFilter::new());
        let upstream_filter = Arc::new(UpstreamtFilter::new());
        self.filter_chain = Arc::new(Mutex::new(FilterChain::new(vec![
            csrf_filter,
            auth_filter,
            logging_filter,
            session_management_filter,
            exception_translation_filter,
            header_writer_filter,
            routing_filter,
            upstream_filter,
        ])));
    }

    pub async fn run(&self) {
        let make_svc = make_service_fn(move |_conn| {
            let filter_chain = self.filter_chain.clone();
            async move {
                Ok::<_, Error>(service_fn(move |req| {
                    let filter_chain = filter_chain.clone();
                    async move {
                        #[derive(Serialize, Deserialize, Debug)]
                        struct Test {
                            name: String,
                        }
                        let end_of_chain: FilterFn = Arc::new(|_req| {
                            let test = Test {
                                name: String::from("jiwan"),
                            };
                            let json_data = serde_json::to_string(&test);
                            //let rsponse = ResponseEntity::build_success(Body::from(json_data));
                            let response = ResponseEntity::build_error(Error::new(
                                sguard_error::ErrorType::ConnectError,
                            ));
                            Box::pin(async move { Ok(response) })
                        });
                        // Build the filter chain in reverse order
                        let next: FilterFn = end_of_chain.clone();

                        let filter_chain = filter_chain.lock().await;
                        let result = filter_chain.handle(&req, next).await;
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
