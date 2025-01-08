use sguard_core::model::route::{Route, RouteDefinition};

use crate::factory::RouteResolver;

pub struct CookieBasedResolver {}

impl RouteResolver for CookieBasedResolver {
    fn resolve(&self, route: &Route) -> RouteDefinition {
        log::debug!("Cookie basd resolver used");
        RouteDefinition{
            id: String::from(route.id.clone()),
            filters: vec![],
            predicates: vec![]
        }   
    }
}