use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Route {
    pub id: String,
    pub uri: String,
    pub predicates: Vec<String>,
    #[serde(default)]
    pub filters: Option<HashMap<String, String>>, // Adjust based on filter structure
}

pub struct RouteBuilder {
    id: Option<String>,
    uri: Option<String>,
    predicates: Vec<String>,
    filters: Option<HashMap<String, String>>,
}

impl RouteBuilder {
    pub fn new() -> Self {
        RouteBuilder {
            id: None,
            uri: None,
            predicates: Vec::new(),
            filters: None,
        }
    }

    pub fn id(mut self, id: &str) -> Self {
        self.id = Some(id.to_string());
        self
    }

    pub fn uri(mut self, uri: &str) -> Self {
        self.uri = Some(uri.to_string());
        self
    }

    pub fn add_predicate(mut self, predicate: &str) -> Self {
        self.predicates.push(predicate.to_string());
        self
    }

    pub fn filters(mut self, filters: HashMap<String, String>) -> Self {
        self.filters = Some(filters);
        self
    }

    pub fn build(self) -> Result<Route, &'static str> {
        if self.id.is_none() {
            return Err("id is required");
        }
        if self.uri.is_none() {
            return Err("uri is required");
        }

        Ok(Route {
            id: self.id.unwrap(),
            uri: self.uri.unwrap(),
            predicates: self.predicates,
            filters: self.filters,
        })
    }
}

#[derive(Clone, Default, Debug)]
pub struct RouteDefinition {
    /// Unique identifier for the route.
    pub id: String,
    /// URL pattern used to match incoming request paths.
    pub uri_pattern: String,
    /// Upstream URL where matching requests will be routed.
    pub uri: String,
    /// Pre-parsed upstream URL (scheme, host, port, base path).
    pub parsed_uri: Option<http::Uri>,
}

impl RouteDefinition {
    pub fn from_route(route: &Route) -> Self {
        let parsed_uri = route.uri.parse::<http::Uri>().ok();
        RouteDefinition {
            id: route.id.clone(),
            uri_pattern: String::from(route.uri.clone()),
            uri: String::from(route.uri.clone()),
            parsed_uri,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub routes: Vec<Route>,
}