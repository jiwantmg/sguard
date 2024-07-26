use std::sync::Arc;

use hyper::{Body, Request};

use crate::core::{Filter, FilterFn, FilterRs};
use crate::filter_chain::FilterChainTrait;

pub trait LoggingFilterTrait: FilterChainTrait {
    fn sub_filter_chain(&self) -> Option<Arc<dyn LoggingFilterTrait>>;
}
pub struct LoggingFilter {
    sub_chain: Option<Arc<dyn LoggingFilterTrait>>,
}

impl LoggingFilter {
    pub fn new(sub_chain: Option<Arc<dyn LoggingFilterTrait>>) -> Self {
        LoggingFilter { sub_chain }
    }
}

impl Filter for LoggingFilter {
    fn handle(&self, req: &Request<Body>, next: FilterFn) -> FilterRs {
        log::debug!("Filter: LoggingFilter");
        panic!("Un recoverable error");
        next(req)
    }
}
