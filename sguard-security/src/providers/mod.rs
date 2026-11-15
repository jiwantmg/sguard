use crate::{core::model::Authentication, errors::AuthError};

pub mod basic_auth_provider;
pub mod jwt_auth_provider;
pub mod open_saml5_auth_provider;
pub mod cookie_auth_provider;

pub trait AuthProvider: Send + Sync {
    fn authenticate(&self, req: &dyn Authentication) -> Result<Box<dyn Authentication>, AuthError>;
}
