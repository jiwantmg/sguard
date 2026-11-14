use crate::basic_auth::BasicAuthentication;
use crate::core::model::Authentication;
use crate::core::source::web::WebAuthDetailSource;
use crate::errors::AuthError;

use super::auth_converter::AuthConverter;

pub struct BasicAuthConverter {
    pub auth_detail_source: WebAuthDetailSource,
}

impl BasicAuthConverter {
    pub fn new(auth_detail_source: WebAuthDetailSource) -> Self {
        Self { auth_detail_source }
    }
}

impl AuthConverter for BasicAuthConverter {
    fn convert(&self, request: &dyn Authentication) -> Result<Box<dyn Authentication>, AuthError> {
        Ok(Box::new(BasicAuthentication::new(String::from("value"))))
    }
}
