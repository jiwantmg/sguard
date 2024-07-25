use crate::core::{Filter, FilterFn, FilterRs};
use hyper::{Body, Request};
use std::sync::Arc;

pub trait FilterChainTrait: Filter {}
pub struct FilterChain {
    filters: Vec<Arc<dyn Filter>>,
}

impl FilterChain {
    pub fn new(filters: Vec<Arc<dyn Filter>>) -> Self {
        FilterChain { filters }
    }
}

impl Filter for FilterChain {
    fn handle(&self, req: &Request<Body>, next: FilterFn) -> FilterRs {
        // Build the filter chain in reverse order
        let mut next = next.clone();

        for filter in self.filters.iter().rev() {
            let current_next = next.clone();
            let filter = filter.clone();
            next = Arc::new(move |req| filter.handle(req, current_next.clone()));
        }

        // Execute the filter chain
        next(req)
    }
}

impl FilterChainTrait for FilterChain {}
