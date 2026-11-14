use crate::{core::model::Authentication, errors::AuthError};

use super::auth_manager_resolver::AuthManager;

pub struct BasicAuthManager {}

impl AuthManager for BasicAuthManager {
    fn authenticate(
        &self,
        authentication: Box<dyn Authentication>,
    ) -> Result<Box<dyn Authentication>, AuthError> {
        println!("Authenticatin via basic way");
        todo!()
    }
}
