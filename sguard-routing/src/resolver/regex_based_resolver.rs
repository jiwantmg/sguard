use crate::{factory::RouteResolver, route::{Route, RouteDefinition}};

pub struct RegexBasedResolver {}
impl RouteResolver for RegexBasedResolver {
    fn resolve(&self, route: &Route) -> RouteDefinition {
        println!("Regex basd resolver used");
        RouteDefinition{
            id: String::from(route.id.clone()),
            filters: vec![],
            predicates: vec![]
        }   
    }
}