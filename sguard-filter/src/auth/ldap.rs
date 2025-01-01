use crate::filter_chain::FilterChainTrait;
use sguard_core::{filter::{Filter, FilterFn, FilterRs}, model::context::RequestContext};
use std::sync::Arc;

use super::AuthFilterTrait;
pub struct SGuardLdapAuth;

impl Filter for SGuardLdapAuth {
    fn handle(&self, req: &mut RequestContext, next: FilterFn) -> FilterRs {
        log::debug!("Filter: Ldap authentications");
        // Perform authentication logic here
        next(req)
    }
}

impl FilterChainTrait for SGuardLdapAuth {}
impl AuthFilterTrait for SGuardLdapAuth {
    fn sub_filter_chain(&self) -> Option<Arc<dyn AuthFilterTrait>> {
        todo!()
    }
}
