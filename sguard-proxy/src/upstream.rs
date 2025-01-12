use std::sync::Arc;
use sguard_core::model::context::RequestContext;
use sguard_error::{Error as AppError, ErrorType};

#[derive(Clone, Default)]
pub struct UpstreamService {}

impl UpstreamService {   
    pub async fn call_upstream_service(&self, req: Arc<RequestContext>) -> Result<String, AppError> {
        let uri = req.as_ref().route_definition.uri.parse::<hyper::Uri>();
        if uri.is_err() {
            return Err(AppError::new(ErrorType::Custom("Invalid url"))); // Early return on error
        }
        Ok(String::from("value"))
    }
}
