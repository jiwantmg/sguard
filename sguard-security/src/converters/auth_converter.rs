use crate::{
    converters::cookie_auth_converter::CookieHttpAuthConverter, core::{model::Authentication, source::web::WebAuthDetailSource}, errors::{AuthError, ErrorType},
};
use hyper::header::{AUTHORIZATION, COOKIE};
use sguard_core::model::core::HttpRequest;

use super::{basic_auth_converter::BasicAuthConverter, jwt_auth_converter::JwtAuthConverter};

pub enum AuthConverterManager {
    Basic(BasicAuthConverter),
    Jwt(JwtAuthConverter),
    Cookie(CookieHttpAuthConverter),
}

impl AuthConverterManager {
    pub fn get_auth_converter(req: &HttpRequest) -> Result<AuthConverterManager, AuthError> {
        if let Some(auth_type) = req.headers.get(AUTHORIZATION) {
            let header_parts: Vec<_> = auth_type.to_str().unwrap().splitn(2, " ").collect();
            if header_parts.is_empty() {
                return Err(AuthError::new(ErrorType::InvalidToken));
            }

            let auth_type_lwr = header_parts[0].to_lowercase();
            log::debug!("Auth type {}", auth_type_lwr);
            match &auth_type_lwr.as_str() {
                &"basic" => Ok(AuthConverterManager::Basic(BasicAuthConverter::new(
                    WebAuthDetailSource {},
                ))),
                &"bearer" => Ok(AuthConverterManager::Jwt(JwtAuthConverter::new(
                    WebAuthDetailSource {},
                ))),
                _ => Err(AuthError::new(ErrorType::UnsupportedAuthentication)),
            }
        } else if req.headers.contains_key(COOKIE) {
            // 2. If Authorization is absent, check Cookie header.
            log::debug!("Cookie authentication requested");
            return Ok(AuthConverterManager::Cookie(
                CookieHttpAuthConverter::new(WebAuthDetailSource {}),
            ));
        
        }
         else {
            Err(AuthError::new(ErrorType::UnknownError))
        }
    }
}

pub trait AuthConverter: Send + Sync {
    fn convert(&self, request: &dyn Authentication) -> Result<Box<dyn Authentication>, AuthError>;
}

// impl AuthConverter<Box<dyn Authentication>> for AuthConverterManager {
//     fn convert(
//         &self,
//         request: &Box<dyn Authentication>,
//     ) -> Result<Box<dyn Authentication>, AuthError> {
//         match self {
//             AuthConverterManager::Basic(converter) => converter.convert(request),
//             AuthConverterManager::Jwt(converter) => converter.convert(request),
//             _ => Err(AuthError {}),
//         }
//     }
//     // fn convert(&self, request: &HttpRequest) -> Result<Box<dyn Authentication>, AuthError> {
//     //     match self {
//     //         AuthConverterManager::Basic(converter) => converter.convert(request),
//     //         AuthConverterManager::Jwt(converter) => converter.convert(request),
//     //         _ => Err(AuthError {}),
//     //     }
//     // }
// }
