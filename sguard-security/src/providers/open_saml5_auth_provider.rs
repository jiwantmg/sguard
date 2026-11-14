use super::AuthProvider;
use crate::{core::model::Authentication, errors::AuthError};

pub struct OpenSaml5AuthProvider {}

impl AuthProvider for OpenSaml5AuthProvider {
    fn authenticate(&self, req: &dyn Authentication) -> Result<Box<dyn Authentication>, AuthError> {
        //Box::new(JwtAuthToken::new(String::from("value")))
        todo!()
    }
}
