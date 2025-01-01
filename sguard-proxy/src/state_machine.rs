use crate::upstream::UpstreamService;
use hyper::{Body, Method, Response};
use sguard_core::{http::ResponseEntity, model::context::RequestContext};
use sguard_error::Error;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

#[derive(Debug)]
enum State {
    Idle,
    Starting,
    Sending,
    Receiving,
    Completed,
    Error,
}

#[derive(Debug)]
pub enum ConnectionEvent {
    Start,
    Send,
    Receive,
    Complete,
    Fail,
}

pub struct StateMachine {
    state: State,
    req: Arc<RequestContext>,
    tx: mpsc::Sender<ConnectionEvent>,
    rx: mpsc::Receiver<ConnectionEvent>,
    on_completed: Option<Box<dyn FnOnce(Response<Body>) + Send>>,
    upstream_service: UpstreamService,
    response: Option<Response<Body>>
}

impl StateMachine {
    pub fn new(
        req: Arc<RequestContext>,
        tx: mpsc::Sender<ConnectionEvent>,
        rx: mpsc::Receiver<ConnectionEvent>,
        on_completed: Option<Box<dyn FnOnce(Response<Body>) + Send>>,
    ) -> Self {
        Self {
            state: State::Idle,
            req,
            tx,
            rx,
            on_completed,
            upstream_service: UpstreamService::new(),
            response: None
        }
    }

    async fn run(&mut self) {
        while let Some(event) = self.rx.recv().await {
            self.handle_event(event).await;
            // Exit loop if in a final state
            if matches!(self.state, State::Completed | State::Error) {
                break;
            }
        }
    }

    pub async fn handle_event(&mut self, event: ConnectionEvent) {
        match self.state {
            State::Idle => match event {
                ConnectionEvent::Start => {
                    log::debug!("Transitioning from Idle to Starting {}", self.req.request.method());                    
                    match self.req.request.method() {
                        &Method::GET => {
                            self.state = State::Starting;
                            self.tx.send(ConnectionEvent::Receive).await.unwrap()
                        }
                        &Method::POST => {
                            self.state = State::Starting;
                            self.tx.send(ConnectionEvent::Send).await.unwrap()
                        }
                        &Method::DELETE => self.tx.send(ConnectionEvent::Send).await.unwrap(),
                        &Method::PUT => self.tx.send(ConnectionEvent::Send).await.unwrap(),
                        _ => self.tx.send(ConnectionEvent::Receive).await.unwrap(),
                    }
                }
                _ => log::error!("Unhandled event in Idle state"),
            },
            State::Starting => match event {
                ConnectionEvent::Send => {
                    log::debug!("Transitioning from Connecting to Sending");
                    self.state = State::Sending;
                    log::debug!("Sending request to upstream {}", self.req.route_definition.id);
                    self.tx.send(ConnectionEvent::Receive).await.unwrap();
                }
                ConnectionEvent::Receive => {
                    log::trace!("Transitioning from Sending to Receiving");
                    self.state = State::Receiving;
                    log::debug!("Get From {}", self.req.route_definition.id);

                    log::debug!("Calling upstream service for {}", self.req.route_definition.id);
                    let response = self.upstream_service.call_upstream_service().await;

                    match response {
                        Ok(response_body) => {
                            self.response = Some(ResponseEntity::build_success(Body::from(response_body)));
                            self.tx.send(ConnectionEvent::Complete).await.unwrap();
                        }
                        Err(_) => {
                            self.response = Some(ResponseEntity::build_error(Error::new(
                                sguard_error::ErrorType::ConnectError,
                            )));
                            self.tx.send(ConnectionEvent::Fail).await.unwrap();
                        }
                    }

                    self.tx.send(ConnectionEvent::Complete).await.unwrap();
                }
                ConnectionEvent::Fail => {
                    log::debug!("Transitioning from Connecting to Error");
                    self.state = State::Error;
                }
                _ => log::error!("Unhandled event in Connecting state"),
            },
            State::Sending => match event {
                ConnectionEvent::Receive => {}
                ConnectionEvent::Fail => {
                    log::debug!("Transitioning from Sending to Error");
                    self.state = State::Error;
                }
                _ => log::error!("Unhandled event in Sending state"),
            },
            State::Receiving => match event {
                ConnectionEvent::Complete => {
                    log::debug!("Transitioning from Receiving to Completed");
                    self.state = State::Completed;
                    // Callback logic moved here
                    if let Some(callback) = self.on_completed.take() {
                        if let Some(response) = self.response.take() {
                            callback(response);
                        } else {
                            log::error!("No response available for callback");
                        }
                    }
                }
                ConnectionEvent::Fail => {
                    log::error!("Transitioning from Receiving to Error");
                    self.state = State::Error;
                }
                _ => log::error!("Unhandled event in Receiving state"),
            },
            State::Completed | State::Error => {
                log::debug!("Final state reached");
            }
            _ => log::error!("Unhandled event in state: {:?}", event),
        }
    }
}

pub struct StateMachineManager {
    next_id: Mutex<usize>,
}

impl StateMachineManager {
    pub fn new() -> Self {
        Self {
            next_id: Mutex::new(0), // Initialize notify
        }
    }

    pub async fn create_state_machine(
        &self,
        req: Arc<RequestContext>,
        response_handler: Option<Box<dyn FnOnce(Response<Body>) + Send>>,
    ) -> Arc<Mutex<StateMachine>> {
        let (tx, rx) = mpsc::channel(10000);
        let mut next_id = self.next_id.lock().await;
        let id = *next_id;
        *next_id += 1;
        let req = req;
        let state_machine = Arc::new(Mutex::new(StateMachine::new(req, tx, rx, response_handler)));
        let sm_clone = state_machine.clone();
        tokio::spawn(async move {
            let mut state_machine = sm_clone.lock().await;
            log::debug!("State machine starting running {}", id);
            state_machine.handle_event(ConnectionEvent::Start).await;
            state_machine.run().await;
            log::debug!("State machine done {}", id);
        });

        log::debug!("Inserting state machine {}", id);
        state_machine.clone()
    }
}
