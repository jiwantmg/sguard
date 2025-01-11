#[derive(Clone, Default)]
pub struct UpstreamService {}

impl UpstreamService {   
    pub async fn call_upstream_service(&self) -> Result<String, hyper::Error> {
        let uri: String = "http://localhost:8090".parse().unwrap();

        // let request = Request::builder()
        //     .method("POST")
        //     .uri(uri)
        //     .header("Content-Type", "application/json")
        //     .body(Body::empty())
        //     .expect("Failed to build request");

        // let response = self.http_client.request(request).await?;
        //let body = hyper::body::to_bytes(response.into_body()).await?;
        let response: Vec<u8> = "Hello, world!".as_bytes().to_vec();
        Ok(String::from_utf8(response).expect("Failed to convert body to string"))
    }
}
