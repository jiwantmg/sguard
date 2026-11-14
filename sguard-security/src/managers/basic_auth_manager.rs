use crate::{core::model::Authentication, errors::AuthError, managers::auth_manager::AuthManager};

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
