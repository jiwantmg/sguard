use crate::core::model::{Authentication, Principal};

pub struct BasicAuthentication {
    authentication: bool,
    token: String,
}

impl BasicAuthentication {
    pub fn new(token: String) -> Self {
        Self {
            token,
            authentication: false,
        }
    }
}

impl Authentication for BasicAuthentication {
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
