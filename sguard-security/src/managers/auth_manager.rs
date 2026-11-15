use crate::{core::model::Authentication, errors::AuthError};

// pub trait AuthManager {
//     /// This function takes authentication object generated from auth_converter
//     /// and return the complete authentication object.
//     ///
//     /// # Arguments
//     ///
//     /// * `auth` - A reference to the HTTP request to check for a match.
//     ///
//     /// # Returns
//     ///
//     /// Returns `Authentication` if the request is valid.
//     /// Returns 'AuthError' if authentication failed
//     fn authenticate(
//         &self,
//         auth: Box<dyn Authentication>,
//     ) -> Result<Box<dyn Authentication>, AuthError>;
// }

pub trait AuthManager: Send + Sync {
    fn authenticate(
        &self,
        auth: Box<dyn Authentication>,
    ) -> Result<Box<dyn Authentication>, AuthError>;
}

// providers/mod.rs
pub trait AuthProvider: Send + Sync {
    fn authenticate(&self, auth: &dyn Authentication) -> Result<Box<dyn Authentication>, AuthError>;
}