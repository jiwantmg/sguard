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

use sguard_error::ErrorType;

use sguard_security::{
    auth,
    core::model::Authentication,
    errors::{AuthError, ErrorType as AuthErrorType},
    http::converters::http_auth_converter::{
        HttpAuthConverter,
        HttpAuthConverterManager,
    },
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
        let converter = HttpAuthConverterManager::get_auth_converter(&req.request)?;
        let authentication = converter.convert(&req.request)?;

        let auth_manager = AuthManagerResolver::resolve(&converter)?;
        let authenticated = auth_manager.authenticate(authentication)?;

        if authenticated.is_authenticated() {
            log::debug!("User is authenticated, you can proceed");
        } else {
            log::debug!("User is not authenticated");
        }
        Ok(authenticated)
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
            let auth_error = auth_result.err().unwrap();
            let response_handler = ErrorTranslationFilter::create_error_handler(
                ErrorType::new( auth_error.etype.as_str(), auth_error.etype.as_code())
            );
            return response_handler(req);
        }
        

        if auth_result.unwrap().is_authenticated() == false {
            log::debug!("User is not authenticated");
            let response_handler =
                ErrorTranslationFilter::create_error_handler(ErrorType::CustomCode(
                    String::from("Unauthorized"),
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
        self.sub_chain.clone()
    }
}
