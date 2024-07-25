use std::future::Future;
use std::pin::Pin;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Error, Request, Response, Server};
//use sguard_filter::auth::{AuthFilter, BasicAuthFilterChain};
use sguard_filter::exception::ExceptionTranslationFilter;
use sguard_filter::filter_chain::FilterChain;
use sguard_filter::http::HeaderWriterFilter;
use sguard_filter::logging::LoggingFilter;
use sguard_filter::security::CsrfFilter;
use sguard_filter::session::SessionManagementFilter;
use std::sync::Arc;
use tokio::sync::Mutex;
use sguard_filter::auth::{AuthFilter, BasicAuthFilterChain};
use sguard_filter::core::Filter;

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
        //let basic_auth_filter = Arc::new(BasicAuthFilterChain::new());
        let csrf_filter = Arc::new(CsrfFilter::new(None));

        let auth_filter = Arc::new(AuthFilter::new(Some(Arc::new(BasicAuthFilterChain))));

        let logging_filter = Arc::new(LoggingFilter::new(None));
        //let logout_filter = Arc::new(LogoutFilter::new(optional_empty_filter_chain.clone()));
        let session_management_filter = Arc::new(SessionManagementFilter::new(None));
        let exception_translation_filter = Arc::new(ExceptionTranslationFilter::new(None));
        let header_writer_filter = Arc::new(HeaderWriterFilter::new(None));

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
                        let end_of_chain: Arc<
                            dyn Fn(
                                &Request<Body>,
                            )
                                -> Pin<Box<dyn Future<Output = Result<Response<Body>, Error>> + Send>>
                            + Send
                            + Sync,
                        > = Arc::new(|_req| Box::pin(async move { Ok(Response::new(Body::from("End of chain"))) }));
                        // Build the filter chain in reverse order
                        let mut next: Arc<dyn Fn(&Request<Body>) -> Pin<Box<dyn Future<Output = Result<Response<Body>, Error>> + Send>> + Send + Sync> = end_of_chain.clone();

                        let filter_chain = filter_chain.lock().await;
                        return filter_chain.handle(&req, next).await;
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
