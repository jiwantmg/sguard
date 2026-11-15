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

#[inline]
fn match_single_pattern(pattern: &str, path: &str) -> bool {
    let pattern = pattern.trim();
    if pattern == path {
        return true;
    }
    if let Some(prefix) = pattern.strip_suffix("/**") {
        let prefix = prefix.trim_end_matches('/');
        return path == prefix
            || path.starts_with(&format!("{}/", prefix))
            || (prefix.is_empty() && path.starts_with('/'));
    }
    if let Some(prefix) = pattern.strip_suffix("/*") {
        let prefix = prefix.trim_end_matches('/');
        if let Some(rest) = path.strip_prefix(prefix) {
            let rest = rest.strip_prefix('/').unwrap_or(rest);
            return !rest.contains('/');
        }
    }
    false
}

#[inline]
fn matches_path(pattern: &str, path: &str) -> bool {
    if pattern.contains(',') {
        pattern.split(',').any(|p| match_single_pattern(p, path))
    } else {
        match_single_pattern(pattern, path)
    }
}

impl Filter for RoutingFilter {
    fn handle(&self, req: &mut RequestContext, next: FilterFn) -> FilterRs {
        let req_path = req.request.uri.path();
        for route_def in &self.routing_definitions {
            if matches_path(&route_def.uri_pattern, req_path) {
                if log::log_enabled!(log::Level::Debug) {
                    log::debug!("Route {} match for {}", route_def.id, req_path);
                }
                req.set_route_definition(route_def.clone());
                return next(req);
            }
        }
        next(req)
    }
}

impl FilterChainTrait for RoutingFilter {}
impl RoutingFilterTrait for RoutingFilter {
    fn sub_filter_chain(&self) -> Option<Arc<dyn RoutingFilterTrait>> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_path_match() {
        assert!(matches_path("/health", "/health"));
        assert!(!matches_path("/health", "/health/check"));
    }

    #[test]
    fn test_wildcard_path_match() {
        assert!(matches_path("/inventory/api/**", "/inventory/api/units"));
        assert!(matches_path("/inventory/api/**", "/inventory/api/units/123/items"));
        assert!(matches_path("/inventory/api/**", "/inventory/api"));
        assert!(!matches_path("/inventory/api/**", "/billing/api"));
    }

    #[test]
    fn test_multi_path_match() {
        let pattern = "/updates/**,/team/api/teams/**";
        assert!(matches_path(pattern, "/updates/live"));
        assert!(matches_path(pattern, "/team/api/teams/42"));
        assert!(!matches_path(pattern, "/other/path"));
    }
}
