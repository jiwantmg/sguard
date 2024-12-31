use crate::{factory::RouteResolver, route::{Route, RouteDefinition}};

pub struct QueryBasedResolver {}
impl RouteResolver for QueryBasedResolver {
    fn resolve(&self, route: &Route) -> RouteDefinition {
        println!("Query basd resolver used");
        RouteDefinition{
            id: String::from(route.id.clone()),
            filters: vec![],
            predicates: vec![]
        }   
    }
}