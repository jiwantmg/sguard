use std::{future::Future, pin::Pin, sync::Arc};

use hyper::{Body, Error, Request, Response};

use crate::{core::Filter, filter_chain::FilterChain};

pub struct AuthFilter {
    sub_chain: Option<Arc<FilterChain>>,
}

impl AuthFilter {
    pub fn new(sub_chain: Option<Arc<FilterChain>>) -> Self {
        AuthFilter { sub_chain }
    }
}

impl Filter for AuthFilter {
    fn handle(
        &self,
        req: &Request<Body>,
        next: Arc<
            (dyn Fn(
                &hyper::Request<Body>,
            ) -> Pin<
                Box<(dyn Future<Output = Result<Response<Body>, hyper::Error>> + Send + 'static)>,
            > + Send
                 + Sync
                 + 'static),
        >,
    ) -> Pin<Box<dyn Future<Output = Result<Response<Body>, Error>> + Send>> {
        log::debug!("Filter: AuthFilter");
        // Perform authentication logic here
        let mut current_next = next.clone();
        if let Some(chain) = &self.sub_chain {
            let chain_handler = Arc::new(Box::new(move |req: &Request<Body>| {
                // Handle the request using the chain
                let current_next = current_next.clone();
                chain.handle(req, Some(current_next))
            }));
            let req_child= &req;
            // Use the `chain_handler` here
            chain_handler(req_child);
        }

        next(req)
    }

    fn sub_filter_chain(&self) -> Option<Arc<FilterChain>> {
        self.sub_chain.clone()
    }
}

pub struct BasicAuthFilterChain {
    sub_chain: Option<Arc<FilterChain>>,
}
impl BasicAuthFilterChain {
    pub fn new() -> Self {
        BasicAuthFilterChain { sub_chain : Some(Arc::new(FilterChain::new(vec![]))) }
    }
}
impl Filter for BasicAuthFilterChain {
    fn handle(
        &self,
        req: &Request<Body>,
        next: Arc<
            dyn Fn(
                    &Request<Body>,
                )
                    -> Pin<Box<dyn Future<Output = Result<Response<Body>, Error>> + Send>>
                + Send
                + Sync,
        >,
    ) -> Pin<Box<dyn Future<Output = Result<Response<Body>, Error>> + Send>> {
        log::debug!("Filter: Basic Aauth filter called");
        // Perform authentication logic here
        let mut current_next = next.clone();
        if let Some(chain) = &self.sub_chain {
            Arc::new(move |req| chain.handle(req, Some(current_next.clone())));
        }
        next(req)
    }
    
    fn sub_filter_chain(&self) -> Option<Arc<FilterChain>> {
        self.sub_chain.clone()
    }
}