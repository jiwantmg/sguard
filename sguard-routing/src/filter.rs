use hyper::{Body, Request};
use sguard_core::filter::{Filter, FilterFn, FilterRs};
use sguard_filter::{filter_chain::FilterChainTrait, routing::RoutingFilterTrait};
use std::sync::Arc;

use crate::route::{load_config, Config};
use crate::factory::{RouteResolverFactory, RouteResolverType};

pub struct RoutingFilter {
    config: Config,
}

impl RoutingFilter {
    pub fn new(file_path: &str) -> Self {
        let routing_filter = RoutingFilter {
            config: load_config(&file_path),
        };

        routing_filter
    }

    pub fn configure_routes(&self) {
        for route in &self.config.routes {
            println!("Route {}", route.id);
            for predicate in &route.predicates {
                let path_array: Vec<&str> = predicate.split("=").collect();
                if path_array.len() >= 2 {
                    let resolver = RouteResolverFactory::get_route_resolver_by_predicate(path_array[0]);
                    resolver.resolve(route);
                }                    
            }
        }
    }
}

impl Filter for RoutingFilter {
    fn handle(&self, req: &Request<Body>, next: FilterFn) -> FilterRs {
        log::debug!("Sub routing filter trait");
        // Perform authentication logic here
        next(req)
    }
}

impl FilterChainTrait for RoutingFilter {}
impl RoutingFilterTrait for RoutingFilter {
    fn sub_filter_chain(&self) -> Option<Arc<dyn RoutingFilterTrait>> {
        todo!()
    }
}
