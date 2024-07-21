use hyper::{Server, Error};
use hyper::service::{make_service_fn, service_fn};
use sguard_filter::filter_chain::FilterChain;
use sguard_filter::middleware::{AuthFilter, LoggingFilter};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct AppBuilder { 
    filter_chain: Arc<Mutex<FilterChain>>,
}

impl AppBuilder {
    pub fn new() -> Self {
        Self {
            filter_chain: Arc::new(Mutex::new(FilterChain::new(vec![])))
        }
    }
    pub fn app_builder(&mut self) {
        let auth_filter = Arc::new(AuthFilter);
        let logging_filter = Arc::new(LoggingFilter);
        self.filter_chain = Arc::new(Mutex::new(FilterChain::new(vec![
            auth_filter,
            logging_filter,
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
                        filter_chain.handle(req).await
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
