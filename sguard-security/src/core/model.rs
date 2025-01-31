use downcast_rs::{impl_downcast, Downcast};

pub trait Authentication: Downcast {
    fn is_authenticated(&self) -> bool;
    fn set_authenticated(&mut self, is_authenticated: bool);
    fn get_principal(&self) -> Box<dyn Principal>;
}

impl_downcast!(Authentication);

pub trait Principal {}

// OAuth2 Authentication
// pub struct OAuth2Authentication {}
// impl Authentication for OAuth2Authentication {
//     fn is_authenticated(&self) -> bool {
//         todo!()
//     }

//     fn set_authenticated(&mut self, _: bool) {}

//     fn get_principal(&self) -> Box<dyn Principal> {
//         todo!()
//     }
// }

// /// OIDC Authentication
// pub struct OIDCAuthentication {}
// impl Authentication for OIDCAuthentication {
//     fn is_authenticated(&self) -> bool {
//         todo!()
//     }

//     fn set_authenticated(&mut self, is_authenticated: bool) {
//         todo!()
//     }

//     fn get_principal(&self) -> Box<dyn Principal> {
//         todo!()
//     }
// }
