use hyper::Response;
use serde_json::json;
use sguard_error::Error;
use sguard_error::ErrorType;

use crate::model::core::HttpResponse;

pub struct ResponseEntity {}

impl ResponseEntity {
    pub fn build_success(data: HttpResponse) -> Response<HttpResponse> {
        Response::builder()
            .status(200) // Set appropriate HTTP status code
            .header("Content-Type", "application/json")
            .body(data)
            .unwrap()
    }
    pub fn build_error(err: Error) -> Response<HttpResponse> {
        // Optionally, you can use `serde_json` to serialize the error message
        let error_message = json!({
            "error": err.to_string(), // Convert the error to a string representation
        });

        Response::builder()
            .status(ErrorType::as_code(&err.etype)) // Set appropriate HTTP status code
            .header("Content-Type", "application/json")
            .body(HttpResponse::default())
            .unwrap()
    }
}
