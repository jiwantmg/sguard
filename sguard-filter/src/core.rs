use hyper::{Request, Response, Body, Error};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub trait Filter: Send + Sync {
    fn handle(
        &self,
        req: Request<Body>,
        next: Arc<dyn Fn(Request<Body>) -> Pin<Box<dyn Future<Output = Result<Response<Body>, Error>> + Send>> + Send + Sync>,
    ) -> Pin<Box<dyn Future<Output = Result<Response<Body>, Error>> + Send>>;
}
