use hyper::{Body, Request, Response, StatusCode};
use serde::{Deserialize, Serialize};
use sguard_core::http::ResponseEntity;
use sguard_error::{Error as SguardError, ErrorType};
use sguard_proxy::state_machine::{ConnectionEvent, StateMachineManager};
use std::sync::Arc;
use tokio::sync::{oneshot, Mutex};

use crate::core::{Filter, FilterFn, FilterRs};
use crate::filter_chain::FilterChainTrait;

pub trait UpstreamFilterTrait: FilterChainTrait {}
pub struct UpstreamtFilter {
    state_machine_manager: Arc<StateMachineManager>,
}

impl UpstreamtFilter {
    pub fn new() -> Self {
        log::debug!("Creating state machine manager");
        UpstreamtFilter {
            state_machine_manager: Arc::new(StateMachineManager::new()),
        }
    }
}

impl Filter for UpstreamtFilter {
    fn handle(&self, req: &Request<Body>, next: FilterFn) -> FilterRs {
        log::debug!("Filter: UpstreamFilter");
        let mut new_request = Request::new(Body::empty());
        *new_request.method_mut() = req.method().clone();
        *new_request.uri_mut() = req.uri().clone();
        *new_request.version_mut() = req.version();
        new_request
            .headers_mut()
            .extend(req.headers().iter().map(|(k, v)| (k.clone(), v.clone())));

        let (tx, rx) = oneshot::channel();
        let state_machine_manager = self.state_machine_manager.clone();
        let req_new = Arc::new(Mutex::new(new_request));

        let response_future = async move {
            tokio::spawn(async move {
                let id = state_machine_manager.create_state_machine(req_new).await;

                let response = if let Some(state_machine) =
                    state_machine_manager.get_state_machine(id).await
                {
                    let mut state_machine = state_machine.lock().await;
                    state_machine.handle_event(ConnectionEvent::Connect).await;
                    let response_guard = state_machine.get_response().await;

                    if let Some(_response) = response_guard.as_ref() {
                        #[derive(Serialize, Deserialize, Debug)]
                        struct Test {
                            name: String,
                        }
                        let test = Test {
                            name: String::from("jiwan"),
                        };
                        let json_data = serde_json::to_string(&test).expect("Failed to serialize");
                        ResponseEntity::build_success(Body::from(json_data))
                    } else {
                        ResponseEntity::build_error(SguardError::new(ErrorType::ConnectError))
                    }
                } else {
                    log::debug!("Something went wrong");
                    ResponseEntity::build_error(SguardError::new(ErrorType::ConnectError))
                };

                let _ = tx.send(response);
            });

            // Await the response from the channel
            match rx.await {
                Ok(result) => Ok(result),
                Err(_) => Err(SguardError::new(ErrorType::ConnectError).into()),
            }
        };

        // Return the boxed future as FilterRs
        log::debug!("1");
        Box::pin(response_future) as FilterRs
    }
}
