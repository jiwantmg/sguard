use std::{future::Future, pin::Pin, sync::Arc};

use hyper::{Body, Error, Request, Response};

use crate::{core::Filter, filter_chain::FilterChain};
pub struct HeaderWriterFilter {
    sub_chain: Option<Arc<FilterChain>>,
}

impl HeaderWriterFilter {
    pub fn new(sub_chain: Option<Arc<FilterChain>>) -> Self {
        HeaderWriterFilter { sub_chain }
    }
}
impl Filter for HeaderWriterFilter {
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
        log::debug!("Filter: HeaderWriterFilter");
        next(req)
    }
    fn sub_filter_chain(&self) -> Option<Arc<FilterChain>> {
        self.sub_chain.clone()
    }
}
