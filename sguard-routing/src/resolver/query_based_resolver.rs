use sguard_core::model::route::{Route, RouteDefinition};

use crate::factory::RouteResolver;

pub struct QueryBasedResolver {}
impl RouteResolver for QueryBasedResolver {
    fn resolve(&self, route: &Route) -> RouteDefinition {
        log::debug!("Query basd resolver used");
        RouteDefinition{
            id: String::from(route.id.clone()),
            filters: vec![],
            predicates: vec![]
        }   
    }
}