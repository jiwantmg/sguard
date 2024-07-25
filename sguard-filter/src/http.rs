use std::sync::Arc;

use hyper::{Body, Request};

use crate::core::{Filter, FilterFn, FilterRs};
use crate::filter_chain::FilterChainTrait;

pub trait HeaderWriterFilterTrait: FilterChainTrait {
    fn sub_filter_chain(&self) -> Option<Arc<dyn HeaderWriterFilterTrait>>;
}
pub struct HeaderWriterFilter {
    sub_chain: Option<Arc<dyn HeaderWriterFilterTrait>>,
}

impl HeaderWriterFilter {
    pub fn new(sub_chain: Option<Arc<dyn HeaderWriterFilterTrait>>) -> Self {
        HeaderWriterFilter { sub_chain }
    }
}
impl Filter for HeaderWriterFilter {
    fn handle(&self, req: &Request<Body>, next: FilterFn) -> FilterRs {
        log::debug!("Filter: HeaderWriterFilter");
        next(req)
    }
}
