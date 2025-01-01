use sguard_core::model::route::{Route, RouteDefinition};

use crate::factory::RouteResolver;

pub struct HostBasedResolver {}

impl RouteResolver for HostBasedResolver {
    fn resolve(&self, route: &Route) -> RouteDefinition {
        log::debug!("Host basd resolver used");
        RouteDefinition{
            id: String::from(route.id.clone()),
            filters: vec![],
            predicates: vec![]
        }   
    }   
}