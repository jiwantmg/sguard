use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Claims {
    pub iss: Option<String>, // The issuer claim (optional if the token doesn't include it)
    pub sub: Option<String>, // Subject (optional)
    pub exp: Option<usize>,  // Expiration timestamp (optional)
    pub aud: Option<String>, // Audience (optional)
}
