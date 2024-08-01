use hyper::{Body, Request, Response};
use serde::{Deserialize, Serialize};
use sguard_core::http::ResponseEntity;
use sguard_error::{Error as SguardError, ErrorType};
use sguard_filter::core::FilterFn;
use sguard_proxy::state_machine::{ConnectionEvent, StateMachineManager};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::{oneshot, Mutex};

pub struct UpstreamService {
    state_machine_manager: Arc<StateMachineManager>,
}

impl UpstreamService {
    pub fn new() -> Self {
        log::debug!("Creating state machine manager");
        UpstreamService {
            state_machine_manager: Arc::new(StateMachineManager::new()),
        }
    }

    pub fn create_handler(&self) -> FilterFn {
        let state_machine_manager = self.state_machine_manager.clone();

        // Update closure to take a reference to Request
        let handler: FilterFn = Arc::new(move |req: &Request<Body>| {
            let state_machine_manager = state_machine_manager.clone();
            let mut new_request = Request::new(Body::empty());
            *new_request.method_mut() = req.method().clone();
            *new_request.uri_mut() = req.uri().clone();
            *new_request.version_mut() = req.version();
            new_request
                .headers_mut()
                .extend(req.headers().iter().map(|(k, v)| (k.clone(), v.clone())));
            let req_new = Arc::new(Mutex::new(new_request));

            let response_future: Pin<
                Box<dyn Future<Output = Result<Response<Body>, Box<sguard_error::Error>>> + Send>,
            > = Box::pin(async move {
                let (tx, rx) = oneshot::channel();

                tokio::spawn(async move {
                    // Define the closure that will handle the response
                    let on_completed = Box::new(|response: Response<Body>| {
                        // Process the response here
                        let _ = tx.send(response);
                    });
                    state_machine_manager
                        .create_state_machine(req_new, Some(on_completed))
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
