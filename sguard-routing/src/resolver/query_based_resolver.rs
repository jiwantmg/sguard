use sguard_core::model::route::{Route, RouteDefinition};

use crate::route::RouteResolver;

pub struct QueryBasedResolver {}
impl RouteResolver for QueryBasedResolver {
    fn resolve(&self, route: &Route) -> RouteDefinition {
        RouteDefinition::from_route(route)  
    }
}