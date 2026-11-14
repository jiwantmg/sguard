use crate::filter_chain::FilterChainTrait;
use hyper::Response;
use sguard_core::{
    filter::{FilterFn, ResponseBody},
    model::context::RequestContext,
};
use sguard_error::{Error, ErrorType, IntoHyperResponse};
use std::sync::Arc;

pub trait ResponseTranslationFilterTrait: FilterChainTrait {
    fn sub_filter_chain(&self) -> Option<Arc<dyn ResponseTranslationFilterTrait>>;
}
pub struct ErrorTranslationFilter {}
impl ErrorTranslationFilter {
    pub fn new() -> Self {
        ErrorTranslationFilter {}
    }
    pub fn create_error_handler(e: ErrorType) -> FilterFn {
        let handler: FilterFn = Arc::new(move |req: &mut RequestContext| {
            Box::pin(async move {
                // Box::pin is essential
                log::error!("Handling error");

                // Create the error response:
                let error = Error::new(ErrorType::InternalError);
                let hyper_response = error.into_hyper_response();
                let response = Response::builder()
                    .status(error.etype.as_code())
                    //.header("Content-Length", body_bytes.len())
                    .body(ResponseBody::Full(hyper_response))
                    .unwrap(); // Handle unwrap appropriately in real code

                Ok(response) // Return the Result
            })
        });
        handler
    }
}
