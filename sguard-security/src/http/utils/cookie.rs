use hyper::{
    header::{COOKIE},
    HeaderMap,
};

/// Extracts the JWT from a named cookie, e.g. `Cookie: token=<jwt>; theme=dark`
pub fn extract_cookie(headers: &HeaderMap, cookie_name: &str) -> Option<String> {
    headers
        .get_all(COOKIE) // a request may contain multiple Cookie headers
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|cookies| cookies.split(';'))
        .find_map(|pair| {
            let (name, value) = pair.trim().split_once('=')?;
            if name == cookie_name && !value.is_empty() {
                Some(value.trim_matches('"').to_string())
            } else {
                None
            }
        })
}