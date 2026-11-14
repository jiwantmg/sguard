use super::auth_converter::AuthConverter;
use crate::auth::jwt::decoder::extract_claim;
use crate::bearer_auth::JwtAuthToken;
use crate::core::model::Authentication;
use crate::core::source::web::WebAuthDetailSource;
use crate::errors::{AuthError, ErrorType};

pub struct JwtAuthConverter {
    auth_detail_source: WebAuthDetailSource,
}

impl JwtAuthConverter {
    pub fn new(auth_detail_source: WebAuthDetailSource) -> Self {
        Self { auth_detail_source }
    }
}

impl AuthConverter for JwtAuthConverter {
    fn convert(&self, request: &dyn Authentication) -> Result<Box<dyn Authentication>, AuthError> {
        if let Some(jwt_auth_token) = request.downcast_ref::<JwtAuthToken>() {
            // parse into claim
            // log::debug!("Claims extracted {:?}", jwt_auth_token.token);
            let claims = extract_claim(&jwt_auth_token.token);
            if claims.is_err() {
                log::debug!("Invalid token {:?}", claims.err());
                return Err(AuthError::new(ErrorType::InvalidToken));
            }

            let sub = claims.unwrap().sub.unwrap();
            // query from db and check if user is valid
            let mut authentication = jwt_auth_token.clone();
            if sub.eq("jiwan.tamang255@gmail.com") {
                authentication.set_authenticated(true);
                // construct a user object and set to details
            }

            Ok(Box::new(authentication))
        } else {
            Err(AuthError::new(
                crate::errors::ErrorType::UnsupportedAuthentication,
            ))
        }
    }
}
