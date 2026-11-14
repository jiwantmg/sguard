use crate::basic_auth::BasicAuthentication;
use crate::core::model::Authentication;
use crate::core::source::web::WebAuthDetailSource;
use crate::errors::AuthError;
use sguard_core::model::core::HttpRequest;

use super::http_auth_converter::HttpAuthConverter;

pub struct BasicHttpAuthConverter {
    pub auth_detail_source: WebAuthDetailSource,
}

impl BasicHttpAuthConverter {
    pub fn new(auth_detail_source: WebAuthDetailSource) -> Self {
        Self { auth_detail_source }
    }
}

impl HttpAuthConverter for BasicHttpAuthConverter {
    fn convert(&self, request: &HttpRequest) -> Result<Box<dyn Authentication>, AuthError> {
        Ok(Box::new(BasicAuthentication::new(String::from("value"))))
    }
}
