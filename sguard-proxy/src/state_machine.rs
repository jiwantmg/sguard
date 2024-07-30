use hyper::{Body, Request, Response};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::MutexGuard;
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
}

impl StateMachine {
    pub fn new(
        req: Arc<Mutex<Request<Body>>>,
        tx: mpsc::Sender<ConnectionEvent>,
        rx: mpsc::Receiver<ConnectionEvent>,
    ) -> Self {
        Self {
            state: State::Idle,
            req,
            tx,
            rx,
            response: Arc::new(Mutex::new(None)),
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
                    println!("Transitioning from Idle to Connecting");
                    self.state = State::Connecting;
                    // Simulate async operation
                    //tokio::time::sleep(Duration::from_secs(1)).await;
                    self.tx.send(ConnectionEvent::Send).await.unwrap();
                }
                _ => println!("Unhandled event in Idle state"),
            },
            State::Connecting => match event {
                ConnectionEvent::Send => {
                    println!("Transitioning from Connecting to Sending");
                    self.state = State::Sending;
                    // Simulate async operation
                    //tokio::time::sleep(Duration::from_secs(1)).await;
                    self.tx.send(ConnectionEvent::Receive).await.unwrap();
                }
                ConnectionEvent::Fail => {
                    println!("Transitioning from Connecting to Error");
                    self.state = State::Error;
                }
                _ => println!("Unhandled event in Connecting state"),
            },
            State::Sending => match event {
                ConnectionEvent::Receive => {
                    println!("Transitioning from Sending to Receiving");
                    self.state = State::Receiving;
                    // Simulate async operation
                    //tokio::time::sleep(Duration::from_secs(1)).await;
                    self.tx.send(ConnectionEvent::Complete).await.unwrap();
                }
                ConnectionEvent::Fail => {
                    println!("Transitioning from Sending to Error");
                    self.state = State::Error;
                }
                _ => println!("Unhandled event in Sending state"),
            },
            State::Receiving => match event {
                ConnectionEvent::Complete => {
                    println!("Transitioning from Receiving to Completed");
                    self.state = State::Completed;
                    let response = Response::new(Body::from("Request processed successfully"));
                    let mut response_lock = self.response.lock().await;
                    *response_lock = Some(response);
                }
                ConnectionEvent::Fail => {
                    println!("Transitioning from Receiving to Error");
                    self.state = State::Error;
                }
                _ => println!("Unhandled event in Receiving state"),
            },
            State::Completed | State::Error => {
                println!("Final state reached");
            }
        }
    }
    pub async fn get_response(&self) -> MutexGuard<Option<Response<Body>>> {
        self.response.lock().await
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
            next_id: Mutex::new(0),
        }
    }

    pub async fn create_state_machine(&self, req: Arc<Mutex<Request<Body>>>) -> usize {
        // Wrap the request in an Arc<Mutex<_>> for shared access
        let (tx, rx) = mpsc::channel(100);
        let mut next_id = self.next_id.lock().await;
        let id = *next_id;
        *next_id += 1;
        let req = req;
        // Initialize the StateMachine with the Arc<Mutex<Request<Body>>>
        let state_machine = Arc::new(Mutex::new(StateMachine::new(req, tx, rx)));
        let sm_clone = state_machine.clone();

        // Spawn a task to run the state machine
        tokio::spawn(async move {
            let mut state_machine = sm_clone.lock().await;
            state_machine.run().await;
        });

        // Insert the state machine into the manager
        let mut state_machines = self.state_machines.lock().await;
        state_machines.insert(id, state_machine);

        id
    }

    pub async fn get_state_machine(&self, id: usize) -> Option<Arc<Mutex<StateMachine>>> {
        let state_machines = self.state_machines.lock().await;
        state_machines.get(&id).cloned()
    }
}

// pub async fn handle_request(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
//     // Handle the HTTP request and simulate state transitions
//     let (tx, _rx) = mpsc::channel(32);
//     let state_machine = StateMachine::new(tx);

//     match req.uri().path() {
//         "/connect" => state_machine.handle_event(ConnectionEvent::Connect).await,
//         "/success" => state_machine.handle_event(ConnectionEvent::Success).await,
//         "/failure" => state_machine.handle_event(ConnectionEvent::Failure).await,
//         "/disconnect" => {
//             state_machine
//                 .handle_event(ConnectionEvent::Disconnect)
//                 .await
//         }
//         "/retry" => state_machine.handle_event(ConnectionEvent::Retry).await,
//         _ => println!("Unknown path"),
//     }

//     Ok(Response::new(Body::from("State transition handled")))
// }
