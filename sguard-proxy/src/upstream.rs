use hyper::{client::HttpConnector, Body, Client, Request};

#[derive(Clone)]
pub struct UpstreamService {
    http_client: Client<HttpConnector, Body>,
}

impl UpstreamService {
    pub fn new() -> Self {
        UpstreamService {
            http_client: Client::new(),
        }
    }

    pub async fn call_upstream_service(&self) -> Result<String, hyper::Error> {
        let uri: String = "http://localhost:8085".parse().unwrap();

        let request = Request::builder()
            .method("POST")
            .uri(uri)
            .header("Content-Type", "application/json")
            .body(Body::empty())
            .expect("Failed to build request");

        let response = self.http_client.request(request).await?;
        let body = hyper::body::to_bytes(response.into_body()).await?;
        Ok(String::from_utf8(body.to_vec()).expect("Failed to convert body to string"))
    }
}
