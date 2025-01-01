use crate::filter_chain::FilterChainTrait;
use sguard_core::{filter::{Filter, FilterFn, FilterRs}, model::context::RequestContext};
use std::sync::Arc;

use super::AuthFilterTrait;
pub struct SGuardBasicAuthFilter;

impl Filter for SGuardBasicAuthFilter {
    fn handle(&self, req: &mut RequestContext, next: FilterFn) -> FilterRs {
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
