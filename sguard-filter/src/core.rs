use hyper::{Body, Error, Request, Response};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// A Filter represents a top level contract that must be shared by all the
/// filters in the application
pub trait Filter: Send + Sync {
    fn handle(&self, req: &Request<Body>, next: FilterFn) -> FilterRs;
}

/// FilterFn is a signature type, instead of writing long type definition for
/// function, we can simply use
/// fn handle(&self, req: &Request<Body>, next: FilterFn) -> FilterRs
pub type FilterFn = Arc<
    dyn Fn(&Request<Body>) -> Pin<Box<dyn Future<Output = Result<Response<Body>, Error>> + Send>>
        + Send
        + Sync,
>;

/// Same as FilterFn, but used for Response instead
pub type FilterRs = Pin<Box<dyn Future<Output = Result<Response<Body>, Error>> + Send>>;
