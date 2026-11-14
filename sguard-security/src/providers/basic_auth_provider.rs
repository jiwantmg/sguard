use super::AuthProvider;
use crate::{bearer_auth::JwtAuthToken, core::model::Authentication, errors::AuthError};

pub struct BasicAuthProvider {}

impl AuthProvider for BasicAuthProvider {
    fn authenticate(&self, req: &dyn Authentication) -> Result<Box<dyn Authentication>, AuthError> {
        //Box::new(JwtAuthToken::new(String::from("value")))
        todo!()
    }
}
