use std::fs;

use sguard_core::model::route::{Config, Route, RouteDefinition};

pub trait RouteResolver {
    fn resolve(&self, config: &Route) -> RouteDefinition;
}

pub trait PathExtractor {
    fn extract(&self, route: &Route) -> Option<String>;
}

pub fn load_config(file_path: &str) -> Config {
    let file_content =
        fs::read_to_string(file_path).unwrap_or_else(|_| panic!("Error while reading config file"));
    // Parse the YAML content into Config
    serde_yaml::from_str(&file_content)
        .unwrap_or_else(|err| panic!("Error occurred while parsing config file {}", err))
}
