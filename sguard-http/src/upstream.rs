use http::Response;
use sguard_core::filter::{FilterFn, FilterRs, ResponseBody};
use sguard_core::model::context::RequestContext;
use sguard_proxy::state_machine::StateMachineManager;
use std::sync::Arc;

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
        let manager = self.state_machine_manager.clone();

        let handler: FilterFn = Arc::new(move |req: &mut RequestContext| {
            let manager = manager.clone();
            let mut req_new = RequestContext {
                route_definition: req.route_definition.clone(),
                request: req.take_request(),
            };

            let response_future: FilterRs = Box::pin(async move {
                // Awaited directly: no spawn, no channel. If the client disconnects,
                // hyper drops this future and the upstream call is cancelled with it.
                manager
                    .forward(&mut req_new)
                    .await
                    .map(|response| Response::new(ResponseBody::Incoming(response)))
            });

            response_future
        });

        handler
    }
}