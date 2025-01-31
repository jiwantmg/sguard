use crate::{
    core::{model::Authentication, source::web::WebAuthDetailSource},
    errors::{AuthError, ErrorType},
};
use hyper::header::AUTHORIZATION;
use sguard_core::model::core::HttpRequest;

use super::{
    basic_http_auth_converter::BasicHttpAuthConverter,
    jwt_http_auth_converter::JwtHttpAuthConverter,
};

pub trait HttpAuthConverter: Send + Sync {
    fn convert(&self, request: &HttpRequest) -> Result<Box<dyn Authentication>, AuthError>;
}

pub enum HttpAuthConverterManager {
    Basic(BasicHttpAuthConverter),
    Jwt(JwtHttpAuthConverter),
}

impl HttpAuthConverterManager {
    pub fn get_auth_converter(req: &HttpRequest) -> Result<HttpAuthConverterManager, AuthError> {
        if let Some(auth_type) = req.headers.get(AUTHORIZATION) {
            let header_parts: Vec<_> = auth_type.to_str().unwrap().splitn(2, " ").collect();
            if header_parts.is_empty() {
                return Err(AuthError::new(ErrorType::UnsupportedAuthentication));
            }

            let auth_type_lwr = header_parts[0].to_lowercase();
            log::debug!("Auth type {}", auth_type_lwr);
            match &auth_type_lwr.as_str() {
                &"basic" => Ok(HttpAuthConverterManager::Basic(
                    BasicHttpAuthConverter::new(WebAuthDetailSource {}),
                )),
                &"bearer" => Ok(HttpAuthConverterManager::Jwt(JwtHttpAuthConverter::new(
                    WebAuthDetailSource {},
                ))),
                _ => Err(AuthError::new(ErrorType::UnsupportedAuthentication)),
            }
        } else {
            Err(AuthError::new(ErrorType::WrongCredentials))
        }
    }
}

impl HttpAuthConverter for HttpAuthConverterManager {
    fn convert(&self, request: &HttpRequest) -> Result<Box<dyn Authentication>, AuthError> {
        match self {
            HttpAuthConverterManager::Basic(converter) => converter.convert(request),
            HttpAuthConverterManager::Jwt(converter) => converter.convert(request),
            _ => Err(AuthError::new(ErrorType::UnsupportedAuthentication)),
        }
    }
}
