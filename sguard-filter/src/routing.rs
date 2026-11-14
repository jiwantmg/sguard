use std::sync::Arc;
use crate::filter_chain::FilterChainTrait;
use sguard_core::{filter::{Filter, FilterFn, FilterRs}, model::context::RequestContext};

// test commit test commit
pub trait RoutingFilterTrait: FilterChainTrait {
    fn sub_filter_chain(&self) -> Option<Arc<dyn RoutingFilterTrait>>;
}

pub struct BaseRoutingFilter {
    sub_chain: Option<Arc<dyn RoutingFilterTrait>>,
}

impl BaseRoutingFilter {
    pub fn new(sub_chain: Option<Arc<dyn RoutingFilterTrait>>) -> Self {
        BaseRoutingFilter { sub_chain }
    }
}
impl Filter for BaseRoutingFilter {
    fn handle(&self, req: &mut RequestContext, next: FilterFn) -> FilterRs {
        log::debug!("Filter: Base Routing");
        // Perform authentication logic here
        let current_next = next.clone();
        if let Some(chain) = &self.sub_chain {
            let chain_handler = Arc::new(Box::new(move |req: &mut RequestContext| {
                // Handle the request using the chain
                let current_next = current_next.clone();
                chain.handle(req, current_next)
            }));
            let req_child = req;
            // Use the `chain_handler` here
            return chain_handler(req_child);
        }
        next(req)
    }
}

impl FilterChainTrait for BaseRoutingFilter {}
impl RoutingFilterTrait for BaseRoutingFilter {
    fn sub_filter_chain(&self) -> Option<Arc<dyn RoutingFilterTrait>> {
        todo!()
    }
}
