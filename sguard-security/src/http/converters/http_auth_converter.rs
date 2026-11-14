use crate::{
    core::{model::Authentication, source::web::WebAuthDetailSource},
    errors::{AuthError, ErrorType},
};
use hyper::header::{AUTHORIZATION, COOKIE};
use sguard_core::model::core::HttpRequest;

use super::{
    basic_http_auth_converter::BasicHttpAuthConverter,
    jwt_http_auth_converter::JwtHttpAuthConverter,
    cookie_http_auth_converter::CookieHttpAuthConverter,
};

pub trait HttpAuthConverter: Send + Sync {
    fn convert(&self, request: &HttpRequest) -> Result<Box<dyn Authentication>, AuthError>;
}

pub enum HttpAuthConverterManager {
    Basic(BasicHttpAuthConverter),
    Jwt(JwtHttpAuthConverter),
    Cookie(CookieHttpAuthConverter),
}

impl HttpAuthConverterManager {
    pub fn get_auth_converter(
        req: &HttpRequest,
    ) -> Result<HttpAuthConverterManager, AuthError> {
        // 1. Check Authorization header first.
        if let Some(auth_header) = req.headers.get(AUTHORIZATION) {
            let auth_value = auth_header
                .to_str()
                .map_err(|_| AuthError::new(ErrorType::WrongCredentials))?;

            let mut parts = auth_value.splitn(2, ' ');

            let auth_scheme = parts.next().unwrap_or_default();
            let credentials = parts.next().unwrap_or_default();

            if credentials.is_empty() {
                return Err(AuthError::new(ErrorType::WrongCredentials));
            }

            log::debug!("Authentication scheme: {}", auth_scheme);

            return match auth_scheme.to_ascii_lowercase().as_str() {
                "basic" => Ok(HttpAuthConverterManager::Basic(
                    BasicHttpAuthConverter::new(WebAuthDetailSource {}),
                )),

                "bearer" => Ok(HttpAuthConverterManager::Jwt(
                    JwtHttpAuthConverter::new(WebAuthDetailSource {}),
                )),
                _ => Err(AuthError::new(
                    ErrorType::UnsupportedAuthentication,
                )),
            };
        }
        // 2. If Authorization is absent, check Cookie header.
        if req.headers.contains_key(COOKIE) {
            log::debug!("Http Cookie authentication requested");
            return Ok(HttpAuthConverterManager::Cookie(
                CookieHttpAuthConverter::new(WebAuthDetailSource {}),
            ));
        }

        println!("No Authorization or Cookie header found");

        // 3. Neither Authorization nor Cookie exists.
        Err(AuthError::new(ErrorType::WrongCredentials))
    }
}

impl HttpAuthConverter for HttpAuthConverterManager {
    fn convert(&self, request: &HttpRequest) -> Result<Box<dyn Authentication>, AuthError> {
        match self {
            HttpAuthConverterManager::Basic(converter) => converter.convert(request),
            HttpAuthConverterManager::Jwt(converter) => converter.convert(request),
            HttpAuthConverterManager::Cookie(converter) => converter.convert(request),
            _ => Err(AuthError::new(ErrorType::UnsupportedAuthentication)),
        }
    }
}
