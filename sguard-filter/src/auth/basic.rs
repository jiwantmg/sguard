use crate::filter_chain::FilterChainTrait;
use hyper::{Body, Request};
use sguard_core::filter::{Filter, FilterFn, FilterRs};
use std::sync::Arc;

use super::AuthFilterTrait;
pub struct SGuardBasicAuthFilter;

impl Filter for SGuardBasicAuthFilter {
    fn handle(&self, req: &Request<Body>, next: FilterFn) -> FilterRs {
        log::debug!("Filter: Basic AuthFilter");
        // Perform authentication logic here
        next(req)
    }
}

impl FilterChainTrait for SGuardBasicAuthFilter {}
impl AuthFilterTrait for SGuardBasicAuthFilter {
    fn sub_filter_chain(&self) -> Option<Arc<dyn AuthFilterTrait>> {
        todo!()
    }
}
