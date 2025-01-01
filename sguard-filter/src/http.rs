use std::sync::Arc;
use crate::filter_chain::FilterChainTrait;
use sguard_core::{filter::{Filter, FilterFn, FilterRs}, model::context::RequestContext};

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
    fn handle(&self, req: &mut RequestContext, next: FilterFn) -> FilterRs {
        log::debug!("Filter: HeaderWriterFilter");
        next(req)
    }
}
