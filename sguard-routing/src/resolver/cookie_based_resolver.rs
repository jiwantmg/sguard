use sguard_core::model::route::{Route, RouteDefinition};

use crate::route::RouteResolver;

pub struct CookieBasedResolver {}

impl RouteResolver for CookieBasedResolver {
    fn resolve(&self, route: &Route) -> RouteDefinition {
        RouteDefinition::from_route(route)
    }
}