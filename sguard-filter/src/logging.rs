use crate::filter_chain::FilterChainTrait;
use sguard_core::{filter::{Filter, FilterFn, FilterRs}, model::context::RequestContext};
use std::sync::Arc;

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
    fn handle(&self, req: &mut RequestContext, next: FilterFn) -> FilterRs {
        log::debug!("Filter: LoggingFilter, PATH: {}", req.request.uri());
        next(req)
    }
}
