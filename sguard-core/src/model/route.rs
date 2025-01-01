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

#[derive(Clone)]
pub struct RouteDefinition {
    pub id: String,
    pub predicates: Vec<String>,
    pub filters: Vec<String>
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub routes: Vec<Route>,
}