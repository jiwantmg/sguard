use hyper::{Body, Error, Request, Response};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use crate::core::Filter;

pub struct FilterChain {
    filters: Vec<Arc<dyn Filter>>,
}

impl FilterChain {
    pub fn new(filters: Vec<Arc<dyn Filter>>) -> Self {
        FilterChain { filters }
    }

    pub fn handle(
        &self,
        req: Request<Body>,
    ) -> Pin<Box<dyn Future<Output = Result<Response<Body>, Error>> + Send>> {
        // Create an initial "end of chain" handler
        let end_of_chain: Arc<
            dyn Fn(
                    Request<Body>,
                )
                    -> Pin<Box<dyn Future<Output = Result<Response<Body>, Error>> + Send>>
                + Send
                + Sync,
        > = Arc::new(|_req| Box::pin(async move { Ok(Response::new(Body::from("End of chain"))) }));

        // Build the filter chain in reverse order
        let mut next = end_of_chain;
        for filter in self.filters.iter().rev() {
            let current_next = next.clone();
            let filter = filter.clone();
            next = Arc::new(move |req| filter.handle(req, current_next.clone()));
        }

        // Execute the filter chain
        next(req)
    }
}
