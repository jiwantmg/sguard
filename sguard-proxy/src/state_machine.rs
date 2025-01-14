use crate::upstream::UpstreamService;
use hyper::{body::Incoming, Method, Response};
use sguard_core::model::context::RequestContext;
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
    Exit,
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
    req: RequestContext,
    tx: mpsc::Sender<ConnectionEvent>,
    rx: mpsc::Receiver<ConnectionEvent>,
    on_completed: Option<Box<dyn FnOnce(Response<Incoming>) + Send>>,
    upstream_service: UpstreamService,
    response: Option<Response<Incoming>>,
}

impl StateMachine {
    pub fn new(
        req: RequestContext,
        tx: mpsc::Sender<ConnectionEvent>,
        rx: mpsc::Receiver<ConnectionEvent>,
        on_completed: Option<Box<dyn FnOnce(Response<Incoming>) + Send>>,
    ) -> Self {
        Self {
            state: State::Idle,
            req,
            tx,
            rx,
            on_completed,
            upstream_service: UpstreamService::default(),
            response: None,
        }
    }

    async fn run(&mut self) {
        while let Some(event) = self.rx.recv().await {
            if matches!(self.state, State::Exit) {
                log::trace!("Exiting from state machine");
                break;
            }
            self.handle_event(event).await;
        }
    }

    pub async fn handle_event(&mut self, event: ConnectionEvent) {
        match self.state {
            State::Idle => match event {
                ConnectionEvent::Start => {
                    log::debug!(
                        "Transitioning from Idle to Starting {}",
                        self.req.request.method
                    );
                    match &self.req.request.method {
                        &Method::GET => {
                            self.state = State::Starting;
                            self.tx.send(ConnectionEvent::Receive).await.unwrap()
                        }
                        &Method::POST => {
                            self.state = State::Sending;
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
                ConnectionEvent::Start => {
                    log::trace!("Transitioning from Connecting to Sending");
                    self.state = State::Sending;
                    log::trace!(
                        "Sending request to upstream {}",
                        self.req.route_definition.uri
                    );
                    if self.req.request.method == "GET" {
                        self.tx.send(ConnectionEvent::Receive).await.unwrap();
                    } else if self.req.request.method == "POST" {
                        self.tx.send(ConnectionEvent::Send).await.unwrap();
                    } else {
                        self.tx.send(ConnectionEvent::Receive).await.unwrap();
                    }
                }
                ConnectionEvent::Receive => {
                    log::trace!("Transitioning from Sending to Receiving");
                    self.state = State::Receiving;
                    log::trace!("Get From {}", self.req.route_definition.id);

                    log::trace!(
                        "Calling upstream service for {}",
                        self.req.route_definition.id
                    );
                    let response = self
                        .upstream_service
                        .call_upstream_service(&mut self.req)
                        .await;

                    match response {
                        Ok(response) => {
                            //self.response = Some(ResponseEntity::build_success(HttpResponse::from(response_body)));
                            self.response = Some(response);                                
                            self.tx.send(ConnectionEvent::Complete).await.unwrap();
                        }
                        Err(_) => {
                            // self.response = Some(ResponseEntity::build_error(Error::new(
                            //     sguard_error::ErrorType::ConnectError,
                            // )));
                            self.tx.send(ConnectionEvent::Fail).await.unwrap();
                        }
                    }

                    self.tx.send(ConnectionEvent::Complete).await.unwrap();
                }
                ConnectionEvent::Fail => {
                    log::trace!("Transitioning from Connecting to Error");
                    self.state = State::Error;
                }
                _ => log::error!("Unhandled event in Connecting state"),
            },
            State::Sending => match event {
                ConnectionEvent::Send => {
                    log::trace!("Transitioning from Sending to Completing");
                    log::trace!("Get From {}", self.req.route_definition.id);
                    log::trace!(
                        "Calling upstream service for {}",
                        self.req.route_definition.id
                    );
                    let response = self
                        .upstream_service
                        .call_upstream_service(&mut self.req)
                        .await;

                    match response {
                        Ok(response) => {
                            // self.response =
                            //     Some(ResponseEntity::build_success(HttpResponse::from(response)));
                            log::debug!("Response {:?}", response.status());
                            self.response = Some(response);
                                //Some(ResponseEntity::build_success(HttpResponse::default()));
                            self.state = State::Completed;
                            self.tx.send(ConnectionEvent::Complete).await.unwrap();
                        }
                        Err(err) => {
                            log::debug!("Error {:?}", err);
                            // self.response = Some(ResponseEntity::build_error(Error::new(
                            //     sguard_error::ErrorType::ConnectError,
                            // )));
                            self.state = State::Error;
                            self.tx.send(ConnectionEvent::Fail).await.unwrap();
                        }
                    }
                }
                _ => log::error!("Unhandled event in Sending state"),
            },
            State::Receiving => match event {
                ConnectionEvent::Fail => {
                    log::error!("Transitioning from Receiving to Error");
                    self.state = State::Error;
                }
                _ => log::error!("Unhandled event in Machine state"),
            },
            State::Completed | State::Error => match event {
                ConnectionEvent::Complete => {
                    // Callback logic moved here
                    log::trace!("Completing state machine");
                    if let Some(callback) = self.on_completed.take() {
                        if let Some(response) = self.response.take() {
                            callback(response);
                        } else {
                            log::error!("No response available for callback");
                        }
                    }
                    self.state = State::Exit;
                    // Exit loop if in a final state
                    log::trace!("Exiting from state machine");
                }
                ConnectionEvent::Fail => {
                    log::trace!("Failed state");
                    self.state = State::Completed;
                    self.tx.send(ConnectionEvent::Complete).await.unwrap();
                }
                _ => {
                    log::debug!("Unhandled final event")
                }
            },
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
        req: RequestContext,
        response_handler: Option<Box<dyn FnOnce(Response<Incoming>) + Send>>,
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
            log::trace!("State machine starting running {}", id);
            state_machine.handle_event(ConnectionEvent::Start).await;
            state_machine.run().await;
            log::trace!("State machine done {}", id);
        });

        log::trace!("Inserting state machine {}", id);
        state_machine.clone()
    }
}
