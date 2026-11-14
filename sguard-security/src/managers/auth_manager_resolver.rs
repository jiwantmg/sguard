use crate::{
    core::model::Authentication,
    errors::{AuthError, ErrorType},
    http::converters::http_auth_converter::HttpAuthConverterManager,
};

use super::{basic_auth_manager::BasicAuthManager, jwt_auth_manager::JwtAuthManager};

pub trait AuthManager {
    /// This function takes authentication object generated from auth_converter
    /// and return the complete authentication object.
    ///
    /// # Arguments
    ///
    /// * `auth` - A reference to the HTTP request to check for a match.
    ///
    /// # Returns
    ///
    /// Returns `Authentication` if the request is valid.
    /// Returns 'AuthError' if authentication failed
    fn authenticate(
        &self,
        auth: Box<dyn Authentication>,
    ) -> Result<Box<dyn Authentication>, AuthError>;
}

pub struct AuthManagerResolver {}

impl AuthManagerResolver {
    pub fn resolve(
        converter: &HttpAuthConverterManager,
    ) -> Result<Box<dyn AuthManager>, AuthError> {
        match &converter {
            HttpAuthConverterManager::Basic(_) => Ok(Box::from(BasicAuthManager {})),
            HttpAuthConverterManager::Jwt(_) => Ok(Box::from(JwtAuthManager::new())),
            _ => Err(AuthError::new(ErrorType::UnsupportedAuthentication)),
        }
    }
}
