use hyper::{Body, Request};
use sguard_proxy::state_machine::{self, ConnectionEvent, StateMachine};
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::core::{Filter, FilterFn, FilterRs};
use crate::filter_chain::FilterChainTrait;

// test commit test commit
pub trait UpstreamFilterTrait: FilterChainTrait {}
pub struct UpstreamtFilter {
    state_machine: Arc<StateMachine>,
}
impl UpstreamtFilter {
    pub fn new() -> Self {
        log::debug!("Creating state machine");
        let (tx, _rx) = mpsc::channel(32);
        UpstreamtFilter {
            state_machine: Arc::new(StateMachine::new(tx)),
        }
    }
}
impl Filter for UpstreamtFilter {
    fn handle(&self, req: &Request<Body>, next: FilterFn) -> FilterRs {
        log::debug!("Filter: UpstreamFilter");

        self.state_machine.handle_event(ConnectionEvent::Connect);
        next(req)
    }
}
