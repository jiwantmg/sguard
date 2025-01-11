use sguard_core::model::route::{Route, RouteDefinition};

use crate::factory::RouteResolver;

pub struct RegexBasedResolver {}
impl RouteResolver for RegexBasedResolver {
    fn resolve(&self, route: &Route) -> RouteDefinition {
        RouteDefinition::from_route(route)  
    }
}