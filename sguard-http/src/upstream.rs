use hyper::{Body, Request, Response};
use sguard_core::filter::FilterFn;
use sguard_core::http::ResponseEntity;
use sguard_core::model::context::RequestContext;
use sguard_core::model::route::RouteDefinition;
use sguard_error::{Error as SguardError, ErrorType};
use sguard_proxy::state_machine::StateMachineManager;
use std::future::Future;
use std::pin::Pin;
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
            let req_new = Arc::new(RequestContext{
                route_definition: req.route_definition.clone(),
                request: hyper::Request::new(Body::empty())
            });

            let response_future: Pin<
                Box<dyn Future<Output = Result<Response<Body>, Box<sguard_error::Error>>> + Send>,
            > = Box::pin(async move {
                let (tx, rx) = oneshot::channel();
                let req_arc_clone = req_new.clone();
                tokio::spawn(async move {
                    // Define the closure that will handle the response
                    let on_completed = Box::new(|response: Response<Body>| {
                        // Process the response here
                        let _ = tx.send(response);
                    });
                    state_machine_manager
                        .create_state_machine(req_arc_clone, Some(on_completed))
                        .await;
                    // log::debug!("State machine id {}", id);
                    // if let Some(state_machine) = state_machine_manager.get_state_machine(id).await {
                    //     let mut state_machine = state_machine.lock().await;
                    //     state_machine.handle_event(ConnectionEvent::Start).await;
                    // } else {
                    //     // log::debug!("State machine not found");
                    //     // ResponseEntity::build_error(SguardError::new(ErrorType::StateMachineError))
                    // };
                });

                // Await the response from the channel
                match rx.await {
                    Ok(result) => Ok(result),
                    Err(_) => Ok(ResponseEntity::build_error(SguardError::new(
                        ErrorType::ConnectError,
                    ))),
                }
            });

            response_future
        });

        handler
    }
}
