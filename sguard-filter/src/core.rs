use hyper::{Body, Error, Request, Response};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub type FilterFn = Arc<
    dyn Fn(&Request<Body>) -> Pin<Box<dyn Future<Output = Result<Response<Body>, Error>> + Send>>
        + Send
        + Sync,
>;
pub type FilterRs = Pin<Box<dyn Future<Output = Result<Response<Body>, Error>> + Send>>;

pub trait Filter: Send + Sync {
    fn handle(&self, req: &Request<Body>, next: FilterFn) -> FilterRs;
}
