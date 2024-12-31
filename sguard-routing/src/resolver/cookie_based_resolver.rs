use crate::{factory::RouteResolver, route::{Route, RouteDefinition}};

pub struct CookieBasedResolver {}

impl RouteResolver for CookieBasedResolver {
    fn resolve(&self, route: &Route) -> RouteDefinition {
        println!("Cookie basd resolver used");
        RouteDefinition{
            id: String::from(route.id.clone()),
            filters: vec![],
            predicates: vec![]
        }   
    }
}