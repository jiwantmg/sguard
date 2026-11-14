use http::{Response, StatusCode};
use sguard_core::filter::{FilterFn, FilterRs, ResponseBody};
use sguard_core::model::context::RequestContext;
use sguard_error::{Error as SguardError, ErrorType};
use sguard_proxy::state_machine::{StateMachineManager, UpstreamResult};
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

        let handler: FilterFn = Arc::new(move |req: &mut RequestContext| {
            let state_machine_manager = state_machine_manager.clone();
            let req_new = RequestContext {
                route_definition: req.route_definition.clone(),
                request: req.clone_request(),
            };

            let response_future: FilterRs = Box::pin(async move {
                let (tx, rx) = oneshot::channel::<UpstreamResult>();

                // create_state_machine already spawns its own task, so no extra spawn here
                state_machine_manager
                    .create_state_machine(
                        req_new,
                        Some(Box::new(move |result: UpstreamResult| {
                            // receiver is gone if the client disconnected; ignore
                            let _ = tx.send(result);
                        })),
                    )
                    .await;

                match rx.await {
                    Ok(Ok(response)) => Ok(Response::new(ResponseBody::Incoming(response))),
                    Ok(Err(status)) => {
                        log::warn!("Upstream failed with {}", status);
                        Err(if status == StatusCode::GATEWAY_TIMEOUT {
                            SguardError::new(ErrorType::ConnectTimeout)
                        } else {
                            SguardError::new(ErrorType::Custom(String::from("Bad gateway")))
                        })
                    }
                    // sender dropped without sending (the task panicked)
                    Err(_) => Err(SguardError::new(ErrorType::ReadError)),
                }
            });

            response_future
        });

        handler
    }
}