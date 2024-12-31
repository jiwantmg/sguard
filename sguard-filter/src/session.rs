use std::sync::Arc;

use hyper::{Body, Request};

use crate::filter_chain::FilterChainTrait;
use sguard_core::filter::{Filter, FilterFn, FilterRs};

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
    fn handle(&self, req: &Request<Body>, next: FilterFn) -> FilterRs {
        log::debug!("Filter: SessionManagementFilter");
        next(req)
    }
}
