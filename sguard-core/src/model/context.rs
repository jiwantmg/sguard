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
        self.take_request()
    }

    pub fn take_request(&mut self) -> HttpRequest {
        HttpRequest {
            parts: self.request.take_parts(),
            incoming: self.request.incoming.take(),
        }
    }

    pub fn set_route_definition(&mut self, route_def: RouteDefinition) {
        self.route_definition = route_def;
    }
}