use super::AuthProvider;
use crate::{
    converters::{auth_converter::AuthConverter, cookie_auth_converter::CookieHttpAuthConverter, jwt_auth_converter::JwtAuthConverter}, core::{model::Authentication, source::web::WebAuthDetailSource}, errors::AuthError,
};

pub struct CookieAuthProvider {
    converter: CookieHttpAuthConverter,
}

impl CookieAuthProvider {
    pub fn new() -> Self {
        Self {
            converter: CookieHttpAuthConverter::new(WebAuthDetailSource {}),
        }
    }
}

impl AuthProvider for CookieAuthProvider {
    fn authenticate(&self, req: &dyn Authentication) -> Result<Box<dyn Authentication>, AuthError> {
        self.converter.convert(req)
    }
}
