use crate::core::{Filter, FilterFn, FilterRs};
use crate::filter_chain::FilterChainTrait;
use hyper::{Body, Request};
use std::sync::Arc;

use super::AuthFilterTrait;

pub struct BasicAuthFilter;

impl Filter for BasicAuthFilter {
    fn handle(&self, req: &Request<Body>, next: FilterFn) -> FilterRs {
        log::debug!("Filter: Basic AuthFilter");
        // Perform authentication logic here
        next(req)
    }
}

impl FilterChainTrait for BasicAuthFilter {}
impl AuthFilterTrait for BasicAuthFilter {
    fn sub_filter_chain(&self) -> Option<Arc<dyn AuthFilterTrait>> {
        todo!()
    }
}
