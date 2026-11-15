use crate::managers::basic_auth_manager::BasicAuthManager;
use crate::managers::jwt_auth_manager::JwtAuthManager;
use crate::managers::cookie_auth_manager::CookieAuthManager;
use crate::managers::auth_manager::AuthManager;
use crate::errors::{AuthError, ErrorType};
use crate::http::converters::http_auth_converter::HttpAuthConverterManager;

pub struct AuthManagerRegistry {
    basic: BasicAuthManager,
    jwt: JwtAuthManager,
    cookie: CookieAuthManager,
}

impl AuthManagerRegistry {
    /// Called once at startup
    pub fn new() -> Self {
        Self {
            basic: BasicAuthManager {},
            jwt: JwtAuthManager::new(),
            cookie: CookieAuthManager::new(),
        }
    }

    /// Called per request: no allocation, just a borrow
    pub fn resolve(
        &self,
        converter: &HttpAuthConverterManager,
    ) -> Result<&dyn AuthManager, AuthError> {
        match converter {
            HttpAuthConverterManager::Basic(_) => Ok(&self.basic),
            HttpAuthConverterManager::Jwt(_) => Ok(&self.jwt),
            HttpAuthConverterManager::Cookie(_) => Ok(&self.cookie),
            _ => Err(AuthError::new(ErrorType::UnsupportedAuthentication)),
        }
    }
}