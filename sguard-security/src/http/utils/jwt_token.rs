use hyper::{header::AUTHORIZATION, HeaderMap};

pub fn extract_bearer_token(headers: &HeaderMap) -> Option<String> {
    // Get the `Authorization` header
    if let Some(auth_header) = headers.get(AUTHORIZATION) {
        // Convert the header value to a string
        if let Ok(auth_value) = auth_header.to_str() {
            // Check if it starts with "Bearer "
            if let Some(token) = auth_value.strip_prefix("Bearer ") {
                // Return the token as a String
                return Some(token.to_string());
            }
        }
    }
    None
}
