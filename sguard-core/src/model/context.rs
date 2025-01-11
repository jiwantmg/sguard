use hyper::Request;
use super::core::HttpRequest;
use super::route::RouteDefinition;

pub struct  RequestContext {
    pub route_definition: RouteDefinition,
    pub request: Request<HttpRequest>
}

impl RequestContext {
    pub fn new(req: &RequestContext) -> Self {
        Self {
            route_definition: req.route_definition.clone(),
            request: Request::new(HttpRequest::default()) // Initialize with empty body, fill later
        }
    }

    pub async fn to_request(&self) -> Request<HttpRequest> {
        let mut req = HttpRequest::default();
        // let body = self.request.body().clone();
        //let bytes = to_bytes(body).await.unwrap_or_default();
        
        // req.headers_mut().extend(self.request.headers().iter().map(|(k,v)| (k.clone(), v.clone())));
        // *req.method_mut() = self.request.method().clone();
        // *req.uri_mut() = self.request.uri().clone();
        // *req.version_mut() = self.request.version();
        // *req.body_mut() = Body::from(Body::empty()); 
        Request::builder().body(req).unwrap()
     }

    pub fn set_route_definition(&mut self, route_def: RouteDefinition) {
        self.route_definition = route_def;
    }
}