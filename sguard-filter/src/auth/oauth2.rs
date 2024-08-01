use crate::core::{Filter, FilterFn, FilterRs};
use crate::filter_chain::FilterChainTrait;
use hyper::{Body, Request};
use std::sync::Arc;

use super::AuthFilterTrait;
pub struct SGuardOAuth2Auth;

impl Filter for SGuardOAuth2Auth {
    fn handle(&self, req: &Request<Body>, next: FilterFn) -> FilterRs {
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
