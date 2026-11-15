use crate::core::model::{Authentication, Principal};

#[derive(Default, Clone)]
pub struct CookieAuthToken {
    authentication: bool,
    pub cookie: String,
}

impl CookieAuthToken {
    pub fn new(cookie: String) -> Self {
        Self {
            cookie,
            authentication: false,
        }
    }
}

impl Authentication for CookieAuthToken {
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
