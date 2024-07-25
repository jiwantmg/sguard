use hyper::service::{make_service_fn, service_fn};
use hyper::{Error, Server};
use sguard_filter::auth::{AuthFilter, BasicAuthFilterChain};
use sguard_filter::exception::ExceptionTranslationFilter;
use sguard_filter::filter_chain::FilterChain;
use sguard_filter::http::HeaderWriterFilter;
use sguard_filter::logging::LoggingFilter;
use sguard_filter::security::CsrfFilter;
use sguard_filter::session::SessionManagementFilter;
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
        let optional_empty_filter_chain = Some(Arc::new(FilterChain::new(vec![])));
        let basic_auth_filter = Arc::new(BasicAuthFilterChain::new());
        let csrf_filter = Arc::new(CsrfFilter::new(Some(Arc::new(FilterChain::new(vec![basic_auth_filter.clone()])))));

        let auth_filter = Arc::new(AuthFilter::new(Some(Arc::new(FilterChain::new(vec![basic_auth_filter.clone()])))));

        let logging_filter = Arc::new(LoggingFilter::new(optional_empty_filter_chain.clone()));
        //let logout_filter = Arc::new(LogoutFilter::new(optional_empty_filter_chain.clone()));
        let session_management_filter = Arc::new(SessionManagementFilter::new(
            optional_empty_filter_chain.clone(),
        ));
        let exception_translation_filter = Arc::new(ExceptionTranslationFilter::new(
            optional_empty_filter_chain.clone(),
        ));
        let header_writer_filter =
            Arc::new(HeaderWriterFilter::new(optional_empty_filter_chain.clone()));
            
        self.filter_chain = Arc::new(Mutex::new(FilterChain::new(vec![
            csrf_filter,
            auth_filter,
            logging_filter,
            session_management_filter,
            exception_translation_filter,
            header_writer_filter,
        ])));
    }

    pub async fn run(&self) {
        let make_svc = make_service_fn(move |_conn| {
            let filter_chain = self.filter_chain.clone();
            async move {
                Ok::<_, Error>(service_fn(move |req| {
                    let filter_chain = filter_chain.clone();
                    async move {
                        let filter_chain = filter_chain.lock().await;
                        filter_chain.handle(&req, None).await
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
