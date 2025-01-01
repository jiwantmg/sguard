use std::sync::Arc;
use crate::filter_chain::FilterChainTrait;
use sguard_core::{filter::{Filter, FilterFn, FilterRs}, model::context::RequestContext};

pub trait SessionManagementFilterTrait: FilterChainTrait {
    fn sub_filter_chain(&self) -> Option<Arc<dyn SessionManagementFilterTrait>>;
}
pub struct SessionManagementFilter {
    sub_chain: Option<Arc<dyn SessionManagementFilterTrait>>,
}
impl SessionManagementFilter {
    pub fn new(sub_chain: Option<Arc<dyn SessionManagementFilterTrait>>) -> Self {
        SessionManagementFilter { sub_chain }
    }
}
impl Filter for SessionManagementFilter {
    fn handle(&self, req: &mut RequestContext, next: FilterFn) -> FilterRs {
        log::debug!("Filter: SessionManagementFilter");
        next(req)
    }
}
