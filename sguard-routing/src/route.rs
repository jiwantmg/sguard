use std::fs;

use sguard_core::model::route::Config;

pub fn load_config(file_path: &str) -> Config {
    let file_content =
        fs::read_to_string(file_path).unwrap_or_else(|_| panic!("Error while reading config file"));
    // Parse the YAML content into Config
    serde_yaml::from_str(&file_content)
        .unwrap_or_else(|err| panic!("Error occurred while parsing config file {}", err))
}
