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
        let handler: FilterFn = Arc::new(move |_req: &mut RequestContext| {
            let error_type = e.clone();
            Box::pin(async move {
                log::error!("Handling error: {:?}", error_type);

                let error = Error::new(error_type);

                // Get status before consuming error.
                let status = error.etype.as_code();

                let hyper_response = error.into_hyper_response();

                let response = Response::builder()
                    .status(status)
                    .body(ResponseBody::Full(hyper_response))
                    .expect("Failed to build error response");

                Ok(response)
            })
        });

        handler
    }
}
