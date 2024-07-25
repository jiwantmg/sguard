use crate::core::{Filter, FilterFn, FilterRs};
use crate::filter_chain::FilterChainTrait;
use hyper::{Body, Request};
use std::sync::Arc;

pub trait CsrfFilterTrait: FilterChainTrait {
    fn sub_filter_chain(&self) -> Option<Arc<dyn CsrfFilterTrait>>;
}
pub struct CsrfFilter {
    sub_chain: Option<Arc<dyn CsrfFilterTrait>>,
}
impl CsrfFilter {
    pub fn new(sub_chain: Option<Arc<dyn CsrfFilterTrait>>) -> Self {
        CsrfFilter { sub_chain }
    }
}
impl Filter for CsrfFilter {
    fn handle(&self, req: &Request<Body>, next: FilterFn) -> FilterRs {
        log::debug!("Filter: CsrfFilter");
        // Perform authentication logic here
        //let mut current_next = next.clone();
        // if let Some(chain) = &self.sub_chain {
        //     Arc::new(move |req| chain.handle(req, Some(current_next.clone())));
        // }
        next(req)
    }
}
