use crate::{factory::RouteResolver, route::{Route, RouteDefinition}};

pub struct PathBasedResolver {}

impl RouteResolver for PathBasedResolver {    
    fn resolve(&self, route: &Route) -> RouteDefinition {
        println!("Path basd resolver used");
        RouteDefinition{
            id: String::from(route.id.clone()),
            filters: vec![],
            predicates: vec![]
        }   
    }
}