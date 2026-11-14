use crate::bearer_auth::JwtAuthToken;
use crate::core::model::Authentication;
use crate::core::source::web::WebAuthDetailSource;
use crate::errors::{AuthError, ErrorType};
use crate::http::utils::jwt_token::extract_bearer_token;
use sguard_core::model::core::HttpRequest;

use super::http_auth_converter::HttpAuthConverter;

pub struct JwtHttpAuthConverter {
    pub auth_detail_source: WebAuthDetailSource,
}

impl JwtHttpAuthConverter {
    pub fn new(auth_detail_source: WebAuthDetailSource) -> Self {
        Self { auth_detail_source }
    }
}

impl HttpAuthConverter for JwtHttpAuthConverter {
    fn convert(&self, request: &HttpRequest) -> Result<Box<dyn Authentication>, AuthError> {
        // build JwtAuthToken from request
        if let Some(token) = extract_bearer_token(&request.headers) {
            Ok(Box::new(JwtAuthToken::new(token)))
        } else {
            Err(AuthError::new(ErrorType::InvalidToken))
        }
    }
}
