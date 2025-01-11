use sguard_core::model::route::{Route, RouteDefinition};

use crate::factory::RouteResolver;

pub struct PathBasedResolver {}

impl RouteResolver for PathBasedResolver {    
    fn resolve(&self, route: &Route) -> RouteDefinition {
        RouteDefinition::from_route(route)   
    }
}