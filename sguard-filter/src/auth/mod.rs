pub mod basic;
use crate::core::{Filter, FilterFn, FilterRs};
use crate::filter_chain::FilterChainTrait;
use hyper::{Body, Request};
use std::sync::Arc;

pub trait AuthFilterTrait: FilterChainTrait {
    fn sub_filter_chain(&self) -> Option<Arc<dyn AuthFilterTrait>>;
}

pub struct AuthFilter {
    sub_chain: Option<Arc<dyn AuthFilterTrait>>,
}

impl AuthFilter {
    pub fn new(sub_chain: Option<Arc<dyn AuthFilterTrait>>) -> Self {
        AuthFilter { sub_chain }
    }
}

impl Filter for AuthFilter {
    fn handle(&self, req: &Request<Body>, next: FilterFn) -> FilterRs {
        log::debug!("Filter: AuthFilter");
        // Perform authentication logic here
        let current_next = next.clone();
        if let Some(chain) = &self.sub_chain {
            let chain_handler = Arc::new(Box::new(move |req: &Request<Body>| {
                // Handle the request using the chain
                let current_next = current_next.clone();
                chain.handle(req, current_next)
            }));
            let req_child = &req;
            // Use the `chain_handler` here
            return chain_handler(req_child);
        }
        next(req)
    }
}

impl FilterChainTrait for AuthFilter {}

impl AuthFilterTrait for AuthFilter {
    fn sub_filter_chain(&self) -> Option<Arc<dyn AuthFilterTrait>> {
        None
    }
}
