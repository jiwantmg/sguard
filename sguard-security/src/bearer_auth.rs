use crate::core::model::{Authentication, Principal};

#[derive(Default, Clone)]
pub struct JwtAuthToken {
    authentication: bool,
    pub token: String,
}

impl JwtAuthToken {
    pub fn new(token: String) -> Self {
        Self {
            token,
            authentication: false,
        }
    }
}

impl Authentication for JwtAuthToken {
    fn is_authenticated(&self) -> bool {
        self.authentication
    }

    fn set_authenticated(&mut self, is_authenticated: bool) {
        self.authentication = is_authenticated
    }

    fn get_principal(&self) -> Box<dyn Principal> {
        todo!()
    }
}
