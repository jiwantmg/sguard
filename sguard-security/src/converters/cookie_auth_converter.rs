use super::auth_converter::AuthConverter;
use crate::cookie_auth::CookieAuthToken;
use crate::core::model::Authentication;
use crate::core::source::web::WebAuthDetailSource;
use crate::errors::{AuthError, ErrorType};

pub struct CookieHttpAuthConverter {
    auth_detail_source: WebAuthDetailSource,
}

impl CookieHttpAuthConverter {
    pub fn new(auth_detail_source: WebAuthDetailSource) -> Self {
        Self { auth_detail_source }
    }
}

impl AuthConverter for CookieHttpAuthConverter {
    fn convert(
        &self,
        request: &dyn Authentication,
    ) -> Result<Box<dyn Authentication>, AuthError> {
        println!("Converting request");
        let cookie_auth = request
            .downcast_ref::<CookieAuthToken>()
            .ok_or_else(|| AuthError::new(ErrorType::UnsupportedAuthentication))?;
        println!("Cookie Auth: {:?}", cookie_auth.cookie);
        let session_id = &cookie_auth.cookie;

        if session_id.is_empty() {
            return Err(AuthError::new(ErrorType::InvalidCookie));
        }

        // TODO: Validate session_id against your session store/database.
        // TODO: Retrieve the associated user.

        let mut authentication = cookie_auth.clone();
        println!("Session ID: {}", session_id);
        // Temporary validation for testing.
        if session_id == "valid-session-id" {
            authentication.set_authenticated(true);
        } else {
            return Err(AuthError::new(ErrorType::InvalidCookie));
        }

        Ok(Box::new(authentication))
    }
}