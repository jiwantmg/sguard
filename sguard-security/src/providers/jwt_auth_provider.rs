use super::AuthProvider;
use crate::{
    converters::{auth_converter::AuthConverter, jwt_auth_converter::JwtAuthConverter},
    core::{model::Authentication, source::web::WebAuthDetailSource},
    errors::AuthError,
};

pub struct JwtAuthProvider {
    converter: JwtAuthConverter,
}

impl JwtAuthProvider {
    pub fn new() -> Self {
        Self {
            converter: JwtAuthConverter::new(WebAuthDetailSource {}),
        }
    }
}

impl AuthProvider for JwtAuthProvider {
    fn authenticate(&self, req: &dyn Authentication) -> Result<Box<dyn Authentication>, AuthError> {
        self.converter.convert(req)
    }
}
