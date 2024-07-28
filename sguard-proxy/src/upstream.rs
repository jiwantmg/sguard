use std::convert::Infallible;

use hyper::{Body, Client, Response};

pub trait UpstreamTrait {
    fn get(&self, url: &str) -> Result<Response<Body>, Infallible>;
}

pub struct Upstream;

impl UpstreamTrait for Upstream {
    fn get(&self, url: &str) -> Result<Response<Body>, Infallible> {
        //     let client = Client::new();
        //     let uri = format!(
        //         "{}{}",
        //         url,
        //         req.uri().path_and_query().map_or("", |x| x.as_str())
        //     );
        //     let mut upstream_req = Request::builder()
        //         .method(req.method())
        //         .uri(uri)
        //         .header(
        //             "Host",
        //             req.headers()
        //                 .get("Host")
        //                 .map_or("", |h| h.to_str().unwrap_or("")),
        //         )
        //         .body(req.into_body())
        //         .expect("Failed to build request");

        //     // Forward the request to the upstream server
        //     let response = client.request(upstream_req).await;
        todo!()
    }
}
