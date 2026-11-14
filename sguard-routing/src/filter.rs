use sguard_core::{filter::{Filter, FilterFn, FilterRs}, model::{context::RequestContext, route::{Config, RouteDefinition}}};
use sguard_filter::{filter_chain::FilterChainTrait, routing::RoutingFilterTrait};
use std::sync::Arc;
use crate::{factory::RouteResolverFactory, route::load_config};

pub struct RoutingFilter {
    config: Config,
    routing_definitions: Vec<RouteDefinition>
}

impl RoutingFilter {
    pub fn new(file_path: &str) -> Self {
        let routing_filter = RoutingFilter {
            config: load_config(&file_path),
            routing_definitions: vec![]
        };
        routing_filter
    }

    pub fn configure_routes(&mut self) {
        let mut route_definitions = Vec::new();
        
        // Iterate over the routes in config and build route definitions
        for route in &self.config.routes {
            for predicate in &route.predicates {
                let path_array: Vec<&str> = predicate.split("=").collect();
                if path_array.len() >= 2 {
                    let resolver = RouteResolverFactory::get_route_resolver_by_predicate(path_array[0]);
                    // Push the resolved route to the new vector
                    route_definitions.push(resolver.resolve(route));
                }
            }
        }

        // Update routing_definitions with the new list of resolved routes
        self.routing_definitions = route_definitions;
    }
}

impl Filter for RoutingFilter {
    fn handle(&self, req: &mut RequestContext, next: FilterFn) -> FilterRs {        
        for route_def in self.routing_definitions.clone() {
            log::debug!("Route {} match for {}", req.request.uri, route_def.uri_pattern);
            if route_def.uri_pattern == req.request.uri.path() {
                req.set_route_definition(route_def);
                return next(req)
            }
        }
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
