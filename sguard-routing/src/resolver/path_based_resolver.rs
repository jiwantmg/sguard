use regex::Regex;
use sguard_core::model::route::{Route, RouteDefinition};
use crate::route::{PathExtractor, RouteResolver};

pub struct PathBasedResolver {}

impl RouteResolver for PathBasedResolver {    
    fn resolve(&self, route: &Route) -> RouteDefinition {        
        let mut route_definition = RouteDefinition::from_route(route);
        // Extract the url_pattern from the config                
        route_definition.uri_pattern = self.extract(route).unwrap();
        log::debug!("Url pattern extracted for {}", route_definition.uri_pattern);
        route_definition
    }
}

impl PathExtractor for PathBasedResolver {
    fn extract(&self, route: &Route) -> Option<String> {
        let path_regex = Regex::new(r"Path=(.*)").unwrap();
        for predicate in &route.predicates {
            if let Some(captures) = path_regex.captures(predicate) {
                return captures.get(1).map(|m| m.as_str().to_string());
            }
        }
        None
    }
}