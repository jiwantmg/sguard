use std::{collections::HashMap, iter::Map};

use sguard_core::model::core::HttpRequest;

pub trait RequestMatcher {
    /// This function determines if the provided HTTP request satisfies the criteria
    /// defined by the specific matching strategy. It performs checks based on various
    /// aspects of the request, such as path, headers, HTTP method, etc., as determined
    /// by the implementation.
    ///
    /// # Arguments
    ///
    /// * `req` - A reference to the HTTP request to check for a match.
    ///
    /// # Returns
    ///
    /// Returns `true` if the request matches the strategy's criteria, and `false` otherwise.
    fn matches(req: &HttpRequest) -> bool;

    fn matcher(req: &HttpRequest);
}

pub struct MatchResult {
    matched: bool,
    variables: HashMap<String, String>,
}

impl MatchResult {
    pub fn new() -> Self {
        Self {
            matched: false,
            variables: HashMap::new(),
        }
    }

    pub fn matched(variables: HashMap<String, String>) -> Self {
        Self {
            variables,
            matched: true,
        }
    }

    pub fn get_variables(&self) -> HashMap<String, String> {
        self.variables.clone()
    }

    pub fn is_matched(&self) -> bool {
        self.matched
    }
}
