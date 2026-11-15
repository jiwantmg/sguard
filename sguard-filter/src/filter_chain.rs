use sguard_core::{
    filter::{Filter, FilterFn, FilterRs},
    model::context::RequestContext,
};
use std::sync::Arc;

pub trait FilterChainTrait: Filter {}
pub struct FilterChain {
    filters: Vec<Arc<dyn Filter>>,
}

impl FilterChain {
    pub fn new(filters: Vec<Arc<dyn Filter>>) -> Self {
        FilterChain { filters }
    }
    /// Compose the whole chain around the terminal handler ONCE, at startup.
    /// The returned handler is then called per request with no allocation.
    pub fn build(&self, terminal: FilterFn) -> FilterFn {
        let mut next: FilterFn = terminal;
        for filter in self.filters.iter().rev() {
            let current_next = next.clone();
            let filter = filter.clone();
            next = Arc::new(move |req: &mut RequestContext| {
                filter.handle(req, current_next.clone())
            });
        }
        next
    }
}

impl Filter for FilterChain {
    fn handle(&self, req: &mut RequestContext, next: FilterFn) -> FilterRs {
        self.build(next)(req)
    }
}

impl FilterChainTrait for FilterChain {}
