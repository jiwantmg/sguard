use std::sync::Arc;
use crate::filter_chain::FilterChainTrait;
use sguard_core::{filter::{Filter, FilterFn, FilterRs}, model::context::RequestContext};

pub trait ExceptionTranslationFilterTrait: FilterChainTrait {
    fn sub_filter_chain(&self) -> Option<Arc<dyn ExceptionTranslationFilterTrait>>;
}
pub struct ExceptionTranslationFilter {
    sub_chain: Option<Arc<dyn ExceptionTranslationFilterTrait>>,
}
impl ExceptionTranslationFilter {
    pub fn new(sub_chain: Option<Arc<dyn ExceptionTranslationFilterTrait>>) -> Self {
        ExceptionTranslationFilter { sub_chain }
    }
}
impl Filter for ExceptionTranslationFilter {
    fn handle(&self, req: &mut RequestContext, next: FilterFn) -> FilterRs {
        log::debug!("Filter: ExceptionTranslationFilter");
        next(req)
    }
}
