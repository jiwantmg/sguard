use sguard_core::model::route::{Route, RouteDefinition};

use crate::resolver::cookie_based_resolver::CookieBasedResolver;
use crate::resolver::host_based_resolver::HostBasedResolver;
use crate::resolver::path_based_resolver::PathBasedResolver;
use crate::resolver::query_based_resolver::QueryBasedResolver;

use crate::resolver::regex_based_resolver::RegexBasedResolver;
pub trait RouteResolver {
    fn resolve(&self, config: &Route) -> RouteDefinition;
}

pub enum RouteResolverType {
    PathBased(PathBasedResolver),
    CookieBased(CookieBasedResolver),
    HostBased(HostBasedResolver),
    QueryBased(QueryBasedResolver),
    RegexBased(RegexBasedResolver)
}

impl RouteResolverType {
    pub fn resolve(&self, route: &Route) -> RouteDefinition {
        match self{
            RouteResolverType::PathBased(resolver) => resolver.resolve(route),
            RouteResolverType::CookieBased(resolver) => resolver.resolve(route),
            RouteResolverType::HostBased(resolver) => resolver.resolve(route),
            RouteResolverType::QueryBased(resolver) => resolver.resolve(route),
            RouteResolverType::RegexBased(resolver) => resolver.resolve(route),
         }
    }
}

pub struct RouteResolverFactory;

impl RouteResolverFactory {
    pub fn get_route_resolver_by_predicate(identifier: &str) -> RouteResolverType {
        match &identifier {
            &"Path" => RouteResolverType::PathBased(PathBasedResolver {  }),
            &"Cookie" => RouteResolverType::CookieBased(CookieBasedResolver {  }),
            &"Host" => RouteResolverType::HostBased(HostBasedResolver {  }),
            &"Query" => RouteResolverType::QueryBased(QueryBasedResolver {  }),
            &"Regex" => RouteResolverType::RegexBased(RegexBasedResolver {  }),
            _ => RouteResolverType::PathBased(PathBasedResolver {})
        }
    }
}