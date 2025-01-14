use super::core::HttpRequest;
use super::route::RouteDefinition;

pub struct  RequestContext {
    pub route_definition: RouteDefinition,
    pub request: HttpRequest
}

impl RequestContext {
    pub fn new(req: &RequestContext) -> Self {
        Self {
            route_definition: req.route_definition.clone(),
            request: HttpRequest::new() // Initialize with empty body, fill later
        }
    }

    pub fn clone_request(&mut self) -> HttpRequest {   
        HttpRequest {
            method: self.request.method.clone(),
            uri: self.request.uri.clone(),
            headers: self.request.headers.clone(),
            incoming: self.request.incoming.take(),
            parts: self.request.parts.take()
        }        
     }

    pub fn set_route_definition(&mut self, route_def: RouteDefinition) {
        self.route_definition = route_def;
    }
}