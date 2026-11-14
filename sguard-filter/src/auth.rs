pub mod basic;
pub mod ldap;
pub mod oauth2;
pub mod saml;

use crate::{
    error::ErrorTranslationFilter, filter_chain::FilterChainTrait,
    response::ResponseTranslationFilter,
};
use hyper::StatusCode;
use sguard_core::{
    filter::{Filter, FilterFn, FilterRs},
    model::context::RequestContext,
};

use sguard_security::{
    core::model::Authentication,
    errors::{AuthError, ErrorType},
    http::converters::http_auth_converter::{HttpAuthConverter, HttpAuthConverterManager},
    managers::auth_manager_resolver::AuthManagerResolver,
};
use std::sync::Arc;

pub trait AuthFilterTrait: FilterChainTrait {
    fn sub_filter_chain(&self) -> Option<Arc<dyn AuthFilterTrait>>;
}

pub struct AuthFilter {
    sub_chain: Option<Arc<dyn AuthFilterTrait>>,
    auth_converter: Option<Box<dyn HttpAuthConverter>>,
}

impl AuthFilter {
    pub fn new(sub_chain: Option<Arc<dyn AuthFilterTrait>>) -> Self {
        AuthFilter {
            sub_chain,
            auth_converter: None,
        }
    }
    pub fn attempt_authentication(
        &self,
        req: &mut RequestContext,
    ) -> Result<Box<dyn Authentication>, AuthError> {
        // extract the authentication type
        let auth_converter_mgr = HttpAuthConverterManager::get_auth_converter(&req.request);
        if auth_converter_mgr.is_err() {
            // todo something
            return Err(AuthError::new(ErrorType::UnsupportedAuthentication));
        }
        let auth_converter_mgr_cloned = auth_converter_mgr.as_ref().clone();
        let authentication = auth_converter_mgr_cloned.unwrap().convert(&req.request);
        let auth_manager = AuthManagerResolver::resolve(auth_converter_mgr_cloned.unwrap());
        let auth_resolved = auth_manager.unwrap().authenticate(authentication.unwrap());
        match &auth_resolved {
            Ok(authentication) => {
                log::debug!("Authentication {:?}", authentication.is_authenticated());
                if authentication.is_authenticated() {
                    log::debug!("User is authenticated, you can proceed");
                } else {
                    log::debug!("User is not authenticated, you can proceed");
                }
                auth_resolved
            }
            Err(_) => Err(AuthError::new(ErrorType::Unauthorized)),
        }
    }
}

impl Filter for AuthFilter {
    fn handle(&self, req: &mut RequestContext, next: FilterFn) -> FilterRs {
        log::debug!("Filter: AuthFilter");
        // Perform authentication logic here
        // authenticate
        let auth_result = self.attempt_authentication(req);

        if auth_result.is_err() {
            //return exception_translation_filter.handle(req, next);
            let response_handler = ErrorTranslationFilter::create_error_handler(
                sguard_error::ErrorType::InternalError,
            );
            return response_handler(req);
        }

        if auth_result.unwrap().is_authenticated() == false {
            log::debug!("User is not authenticated");
            let response_handler =
                ErrorTranslationFilter::create_error_handler(sguard_error::ErrorType::CustomCode(
                    "UnAuthorized",
                    StatusCode::UNAUTHORIZED.as_u16(),
                ));
            return response_handler(req);
        }

        if let Some(chain) = &self.sub_chain {
            let chain_handler = Arc::new(Box::new(move |req: &mut RequestContext| {
                // Handle the request using the chain
                let current_next = next.clone();
                chain.handle(req, current_next)
            }));
            let req_child = req;
            // Use the `chain_handler` here
            return chain_handler(req_child);
        }
        next(req)
    }
}

impl FilterChainTrait for AuthFilter {}

impl AuthFilterTrait for AuthFilter {
    fn sub_filter_chain(&self) -> Option<Arc<dyn AuthFilterTrait>> {
        None
    }
}
