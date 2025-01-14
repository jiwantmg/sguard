use hyper::body::Incoming;
use hyper::Response;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use sguard_error::Error;
use crate::model::context::RequestContext;

/// A Filter represents a top level contract that must be shared by all the
/// filters in the application
pub trait Filter: Send + Sync {
    fn handle(&self, req: &mut RequestContext, next: FilterFn) -> FilterRs;
}

pub type FilterFn = Arc<
    dyn Fn(
            &mut RequestContext,
        ) -> FilterRs
        + Send
        + Sync,
>;
/// Same as FilterFn, but used for Response instead
pub type FilterRs = Pin<Box<dyn Future<Output = Result<Response<Incoming>, Error>> + Send>>;