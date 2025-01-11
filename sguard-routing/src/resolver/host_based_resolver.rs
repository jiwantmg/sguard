use sguard_core::model::route::{Route, RouteDefinition};

use crate::factory::RouteResolver;

pub struct HostBasedResolver {}

impl RouteResolver for HostBasedResolver {
    fn resolve(&self, route: &Route) -> RouteDefinition {
        RouteDefinition::from_route(route)  
    }   
}