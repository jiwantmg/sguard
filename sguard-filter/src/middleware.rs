use crate::core::Filter;
use hyper::{Body, Error, Request, Response};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
pub struct AuthFilter;

impl Filter for AuthFilter {
    fn handle(
        &self,
        req: Request<Body>,
        next: Arc<
            (dyn Fn(
                hyper::Request<Body>,
            ) -> Pin<
                Box<(dyn Future<Output = Result<Response<Body>, hyper::Error>> + Send + 'static)>,
            > + Send
                 + Sync
                 + 'static),
        >,
    ) -> Pin<Box<dyn Future<Output = Result<Response<Body>, Error>> + Send>> {
        log::info!("Authenticating request...");
        // Perform authentication logic here
        next(req)
    }
}

pub struct LoggingFilter;

impl Filter for LoggingFilter {
    fn handle(
        &self,
        req: Request<Body>,
        next: Arc<
            (dyn Fn(
                hyper::Request<Body>,
            ) -> Pin<
                Box<(dyn Future<Output = Result<Response<Body>, hyper::Error>> + Send + 'static)>,
            > + Send
                 + Sync
                 + 'static),
        >,
    ) -> Pin<Box<dyn Future<Output = Result<Response<Body>, Error>> + Send>> {
        log::info!(
            "Method: {}, path: {}",
            req.method().to_string(),
            req.uri().to_string()
        );
        next(req)
    }
}
