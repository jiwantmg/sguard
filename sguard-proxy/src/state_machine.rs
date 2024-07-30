use hyper::{Body, Request, Response};
use serde::{Deserialize, Serialize};
use sguard_core::http::ResponseEntity;
use sguard_error::Error;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{mpsc, Mutex};

#[derive(Debug)]
enum State {
    Idle,
    Connecting,
    Sending,
    Receiving,
    Completed,
    Error,
}

#[derive(Debug)]
pub enum ConnectionEvent {
    Connect,
    Send,
    Receive,
    Complete,
    Fail,
}

pub struct StateMachine {
    state: State,
    req: Arc<Mutex<Request<Body>>>,
    tx: mpsc::Sender<ConnectionEvent>,
    rx: mpsc::Receiver<ConnectionEvent>,
    response: Arc<Mutex<Option<Response<Body>>>>,
    on_completed: Option<Box<dyn FnOnce(Response<Body>) + Send>>,
}

impl StateMachine {
    pub fn new(
        req: Arc<Mutex<Request<Body>>>,
        tx: mpsc::Sender<ConnectionEvent>,
        rx: mpsc::Receiver<ConnectionEvent>,
        on_completed: Option<Box<dyn FnOnce(Response<Body>) + Send>>,
    ) -> Self {
        Self {
            state: State::Idle,
            req,
            tx,
            rx,
            response: Arc::new(Mutex::new(None)),
            on_completed,
        }
    }

    async fn run(&mut self) {
        while let Some(event) = self.rx.recv().await {
            self.handle_event(event).await;
        }
    }

    pub async fn handle_event(&mut self, event: ConnectionEvent) {
        match self.state {
            State::Idle => match event {
                ConnectionEvent::Connect => {
                    log::debug!("Transitioning from Idle to Connecting");
                    self.state = State::Connecting;
                    self.tx.send(ConnectionEvent::Send).await.unwrap();
                }
                _ => log::error!("Unhandled event in Idle state"),
            },
            State::Connecting => match event {
                ConnectionEvent::Send => {
                    log::debug!("Transitioning from Connecting to Sending");
                    self.state = State::Sending;
                    self.tx.send(ConnectionEvent::Receive).await.unwrap();
                }
                ConnectionEvent::Fail => {
                    log::debug!("Transitioning from Connecting to Error");
                    self.state = State::Error;
                }
                _ => log::error!("Unhandled event in Connecting state"),
            },
            State::Sending => match event {
                ConnectionEvent::Receive => {
                    log::debug!("Transitioning from Sending to Receiving");
                    self.state = State::Receiving;
                    self.tx.send(ConnectionEvent::Complete).await.unwrap();
                }
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
                    #[derive(Serialize, Deserialize, Debug)]
                    struct Test {
                        name: String,
                    }
                    let test = Test {
                        name: String::from("jiwan"),
                    };

                    let json_data = match serde_json::to_string(&test) {
                        Ok(json) => json,
                        Err(_) => String::from("Can not convert to string"),
                    };
                    let response = ResponseEntity::build_success(Body::from(json_data));
                    // let response = ResponseEntity::build_error(Error::new(
                    //     sguard_error::ErrorType::ConnectError,
                    // ));

                    if let Some(callback) = self.on_completed.take() {
                        // Call the closure with the response
                        callback(response);
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
        }
    }
}

pub struct StateMachineManager {
    state_machines: Mutex<HashMap<usize, Arc<Mutex<StateMachine>>>>,
    next_id: Mutex<usize>,
}

impl StateMachineManager {
    pub fn new() -> Self {
        Self {
            state_machines: Mutex::new(HashMap::new()),
            next_id: Mutex::new(0), // Initialize notify
        }
    }

    pub async fn create_state_machine(
        &self,
        req: Arc<Mutex<Request<Body>>>,
        response_handler: Option<Box<dyn FnOnce(Response<Body>) + Send>>,
    ) -> usize {
        let (tx, rx) = mpsc::channel(100);
        let mut next_id = self.next_id.lock().await;
        let id = *next_id;
        *next_id += 1;
        let req = req;
        let state_machine = Arc::new(Mutex::new(StateMachine::new(req, tx, rx, response_handler)));
        let sm_clone = state_machine.clone();

        tokio::spawn(async move {
            let mut state_machine = sm_clone.lock().await;
            state_machine.run().await;
        });

        let mut state_machines = self.state_machines.lock().await;
        state_machines.insert(id, state_machine);
        id
    }

    pub async fn get_state_machine(&self, id: usize) -> Option<Arc<Mutex<StateMachine>>> {
        let state_machines = self.state_machines.lock().await;
        state_machines.get(&id).cloned()
    }
}
