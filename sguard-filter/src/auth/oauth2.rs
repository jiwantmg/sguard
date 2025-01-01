use crate::filter_chain::FilterChainTrait;
use sguard_core::{filter::{Filter, FilterFn, FilterRs}, model::context::RequestContext};
use std::sync::Arc;

use super::AuthFilterTrait;
pub struct SGuardOAuth2Auth;

impl Filter for SGuardOAuth2Auth {
    fn handle(&self, req: &mut RequestContext, next: FilterFn) -> FilterRs {
        log::debug!("Filter: OAuth2");
        // Perform authentication logic here
        next(req)
    }
}

impl FilterChainTrait for SGuardOAuth2Auth {}
impl AuthFilterTrait for SGuardOAuth2Auth {
    fn sub_filter_chain(&self) -> Option<Arc<dyn AuthFilterTrait>> {
        todo!()
    }
}
