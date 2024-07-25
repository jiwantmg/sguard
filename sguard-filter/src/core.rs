use hyper::{Body, Error, Request, Response};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use crate::filter_chain::FilterChain;

pub trait Filter: Send + Sync {
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
    ) -> Pin<Box<dyn Future<Output = Result<Response<Body>, Error>> + Send>>;
    
    fn sub_filter_chain(&self) -> Option<Arc<FilterChain>>;
}
