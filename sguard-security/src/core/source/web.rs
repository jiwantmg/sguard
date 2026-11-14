use sguard_core::model::core::HttpRequest;

use super::auth_detail_source::{AuthDetail, AuthDetailSource};

pub struct WebAuthDetail {}
impl AuthDetail for WebAuthDetail {}

pub struct WebAuthDetailSource {}

impl AuthDetailSource<WebAuthDetail, HttpRequest> for WebAuthDetailSource {
    fn build_detail(context: HttpRequest) -> WebAuthDetail {
        println!("Building WebAuthDetail from HttpRequest");
        todo!()
    }
}
