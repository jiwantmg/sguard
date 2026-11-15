use crate::{
    core::model::Authentication, errors::AuthError, managers::auth_manager::AuthManager, providers::{AuthProvider, jwt_auth_provider::JwtAuthProvider},
};

pub struct JwtAuthManager {
    provider: Box<dyn AuthProvider>,
}

impl JwtAuthManager {
    pub fn new() -> Self {
        Self {
            provider: Box::from(JwtAuthProvider::new()),
        }
    }
}

impl AuthManager for JwtAuthManager {
    fn authenticate(
        &self,
        auth: Box<dyn Authentication>,
    ) -> Result<Box<dyn Authentication>, AuthError> {
        self.provider.authenticate(&*auth)
    }
}
