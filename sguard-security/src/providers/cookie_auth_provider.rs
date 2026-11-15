use super::AuthProvider;
use crate::{
    converters::{auth_converter::AuthConverter, cookie_auth_converter::CookieAuthConverter}, core::{model::Authentication, source::web::WebAuthDetailSource}, errors::AuthError,
};

pub struct CookieAuthProvider {
    converter: CookieAuthConverter,
}

impl CookieAuthProvider {
    pub fn new() -> Self {
        Self {
            converter: CookieAuthConverter::new(WebAuthDetailSource {}),
        }
    }
}

impl AuthProvider for CookieAuthProvider {
    fn authenticate(&self, req: &dyn Authentication) -> Result<Box<dyn Authentication>, AuthError> {
        self.converter.convert(req)
    }
}
