use hyper::{Body, Request};

use crate::core::{Filter, FilterFn, FilterRs};
use crate::filter_chain::FilterChainTrait;

// test commit test commit
pub trait RoutingFilterTrait: FilterChainTrait {}
pub struct RoutingFilter {}

impl RoutingFilter {
    pub fn new() -> Self {
        RoutingFilter {}
    }
}
impl Filter for RoutingFilter {
    fn handle(&self, req: &Request<Body>, next: FilterFn) -> FilterRs {
        log::debug!("Filter: RoutingFilter");
        next(req)
    }
}
