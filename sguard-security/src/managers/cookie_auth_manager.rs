use crate::{
    core::model::Authentication, errors::AuthError, providers::{AuthProvider, cookie_auth_provider::CookieAuthProvider},
};

use super::auth_manager_resolver::AuthManager;

pub struct CookieAuthManager {
    provider: Box<dyn AuthProvider>,
}

impl CookieAuthManager {
    pub fn new() -> Self {
        Self {
            provider: Box::from(CookieAuthProvider::new()),
        }
    }
}

impl AuthManager for CookieAuthManager {
    fn authenticate(
        &self,
        auth: Box<dyn Authentication>,
    ) -> Result<Box<dyn Authentication>, AuthError> {
        self.provider.authenticate(&*auth)
    }
}
