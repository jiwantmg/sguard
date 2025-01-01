use crate::filter_chain::FilterChainTrait;
use sguard_core::{filter::{Filter, FilterFn, FilterRs}, model::context::RequestContext};
use std::sync::Arc;

use super::AuthFilterTrait;
pub struct SGuardSamlAuth;

impl Filter for SGuardSamlAuth {
    fn handle(&self, req: &mut RequestContext, next: FilterFn) -> FilterRs {
        log::debug!("Filter: Ldap authentications");
        // Perform authentication logic here
        next(req)
    }
}

impl FilterChainTrait for SGuardSamlAuth {}
impl AuthFilterTrait for SGuardSamlAuth {
    fn sub_filter_chain(&self) -> Option<Arc<dyn AuthFilterTrait>> {
        todo!()
    }
}
