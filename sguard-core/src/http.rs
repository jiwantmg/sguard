use hyper::Body;
use hyper::Response;
use serde_json::json;
use sguard_error::Error;
use sguard_error::ErrorType;

pub struct ResponseEntity {}

impl ResponseEntity {
    pub fn build_success(data: Body) -> Response<Body> {
        todo!()
    }
    pub fn build_error(err: Box<Error>) -> Response<Body> {
        // Optionally, you can use `serde_json` to serialize the error message
        let error_message = json!({
            "error": err.to_string(), // Convert the error to a string representation
        });

        Response::builder()
            .status(ErrorType::as_code(&err.etype)) // Set appropriate HTTP status code
            .header("Content-Type", "application/json")
            .body(Body::from(error_message.to_string()))
            .unwrap()
    }
}
