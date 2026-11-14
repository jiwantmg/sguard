use crate::upstream::UpstreamService;
use hyper::{body::Incoming, Response, StatusCode};
use sguard_core::model::context::RequestContext;
use sguard_error::ErrorType;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

/// What the callback receives: the upstream response, or the HTTP status
/// the gateway should answer with (502/504).
pub type UpstreamResult = Result<Response<Incoming>, StatusCode>;
pub type ResponseHandler = Box<dyn FnOnce(UpstreamResult) + Send>;

pub struct StateMachineManager {
    next_id: AtomicUsize,
    /// One client and connection pool shared by every request.
    upstream: Arc<UpstreamService>,
}

impl StateMachineManager {
    /// Create ONCE at startup and share via Arc. Creating it per request
    /// throws the connection pool away.
    pub fn new() -> Self {
        Self {
            next_id: AtomicUsize::new(0),
            upstream: Arc::new(UpstreamService::new()),
        }
    }

    /// Runs the upstream call in a background task and invokes `response_handler`
    /// exactly once, with either the response or an error status.
    pub async fn create_state_machine(
        &self,
        req: RequestContext,
        response_handler: Option<ResponseHandler>,
    ) {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let upstream = self.upstream.clone();

        tokio::spawn(async move {
            let mut req = req;
            log::trace!("Request {} -> upstream {}", id, req.route_definition.id);

            let result: UpstreamResult = match upstream.call_upstream_service(&mut req).await {
                Ok(response) => {
                    log::trace!("Request {} upstream status {}", id, response.status());
                    Ok(response)
                }
                Err(err) => {
                    log::warn!("Request {} upstream error: {:?}", id, err);
                    Err(match err.etype {
                        ErrorType::ConnectTimeout => StatusCode::GATEWAY_TIMEOUT, // 504
                        _ => StatusCode::BAD_GATEWAY,                             // 502
                    })
                }
            };

            match response_handler {
                Some(callback) => callback(result),
                None => log::trace!("Request {} finished with no handler", id),
            }
        });
    }
}

impl Default for StateMachineManager {
    fn default() -> Self {
        Self::new()
    }
}