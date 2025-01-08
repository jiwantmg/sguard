use sguard_core::{filter::{Filter, FilterFn, FilterRs}, model::context::RequestContext};
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
    fn handle(&self, req: &mut RequestContext, next: FilterFn) -> FilterRs {
        let mut next: Arc<dyn Fn(&mut RequestContext) -> FilterRs + Send + Sync> = next.clone();
        // Build the filter chain in reverse order. Building the filter chain in reverse
        // order is a common pattern in middleware and filter frameworks, particularly in web
        // servers and processing pipelines. Here’s why this approach is used
        // Why Build in Reverse Order?
        // 1. Chain Execution Order:
        //      a. When you build the chain in reverse order, you ensure that filters are applied
        //         in the order you want them to be executed.
        //      b. For example, if you have filters A, B, and C, and you want them to process requests
        //         in the order A -> B -> C, building the chain in reverse (starting from C and adding
        //         A last) ensures that requests pass through A first and then proceed to B and C.
        // 2. Handler Composition:
        //      a. Each filter typically wraps the next filter in the chain. If you start building the chain
        //         from the end (i.e., from the filter that should handle the request last), each filter can
        //         wrap the previous filter's handler.
        //      b. By the time you finish building the chain, the first filter you added (A) will be the
        //         outermost filter, and the last filter you added (C) will be the innermost filter.
        for filter in self.filters.iter().rev() {
            let current_next = next.clone();
            let filter = filter.clone();
            next = Arc::new(move |req: &mut RequestContext| {
                filter.handle(req, current_next.clone())
            });
        }
        next(req)
    }
}

impl FilterChainTrait for FilterChain {}
