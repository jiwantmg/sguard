use crate::cookie_auth::CookieAuthToken;
use crate::core::model::Authentication;
use crate::core::source::web::WebAuthDetailSource;
use crate::errors::{AuthError, ErrorType};
use crate::http::utils::cookie::extract_cookie;
use sguard_core::model::core::HttpRequest;

use super::http_auth_converter::HttpAuthConverter;
const AUTH_COOKIE_NAME: &str = "session";
pub struct CookieHttpAuthConverter {
    pub auth_detail_source: WebAuthDetailSource,
}
    
impl CookieHttpAuthConverter {
    pub fn new(auth_detail_source: WebAuthDetailSource) -> Self {
        Self { auth_detail_source }
    }
}

impl HttpAuthConverter for CookieHttpAuthConverter {
    fn convert(&self, request: &HttpRequest) -> Result<Box<dyn Authentication>, AuthError> {
        // build CookieAuthToken from request
        if let Some(cookie) = extract_cookie(&request.headers, AUTH_COOKIE_NAME) {
            Ok(Box::new(CookieAuthToken::new(cookie)))
        } else {
            Err(AuthError::new(ErrorType::InvalidCookie))
        }
    }
}

