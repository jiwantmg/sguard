use http::Response;
use hyper::body::Incoming;
use sguard_core::filter::{FilterFn, FilterRs};
use sguard_core::model::context::RequestContext;
use sguard_error::{Error as SguardError, ErrorType};
use sguard_proxy::state_machine::StateMachineManager;
use std::sync::Arc;
use tokio::sync::oneshot;

pub struct UpstreamService {
    state_machine_manager: Arc<StateMachineManager>,
}

impl UpstreamService {
    pub fn new(state_manager: Arc<StateMachineManager>) -> Self {
        log::debug!("Creating state machine manager");
        UpstreamService {
            state_machine_manager: state_manager,
        }
    }

    pub fn create_handler(&self) -> FilterFn {
        let state_machine_manager = self.state_machine_manager.clone();
        // Update closure to take a reference to Request
        let handler: FilterFn = Arc::new(move |req: &mut RequestContext| {
            let state_machine_manager = state_machine_manager.clone();
            // *new_request.method_mut() = req.request.method().clone();
            // *new_request.uri_mut() = req.request.uri().clone();
            // *new_request.version_mut() = req.request.version();
            // new_request
            //     .headers_mut()
            //     .extend(req.request.headers().iter().map(|(k, v)| (k.clone(), v.clone())));
            // let req_new = Arc::new(new_request);
            let req_new = RequestContext {
                route_definition: req.route_definition.clone(),
                request: req.clone_request(),
            };

            let response_future: FilterRs = Box::pin(async move {
                let (tx, rx) = oneshot::channel();
                tokio::spawn(async move {
                    // Define the closure that will handle the response
                    let on_completed = Box::new(|response: Response<Incoming>| {
                        // Process the response here
                        let _ = tx.send(response);
                    });
                    state_machine_manager
                        .create_state_machine(req_new, Some(on_completed))
                        .await;
                });

                // Await the response from the channel
                match rx.await {
                    Ok(result) => Ok(result),
                    Err(_) => Err(SguardError::new(ErrorType::ReadError)),
                }
            });

            response_future
        });

        handler
    }
}
