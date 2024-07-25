use std::{future::Future, pin::Pin, sync::Arc};

use hyper::{Body, Error, Request, Response};

use crate::{core::Filter, filter_chain::FilterChain};

pub struct CsrfFilter {
    sub_chain: Option<Arc<FilterChain>>,
}
impl CsrfFilter {
    pub fn new(sub_chain: Option<Arc<FilterChain>>) -> Self {
        CsrfFilter { sub_chain }
    }
}
impl Filter for CsrfFilter {
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
        log::debug!("Filter: CsrfFilter");
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
