use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

#[derive(Debug)]
enum State {
    Idle,
    Connecting,
    Connected,
    Error,
}

pub struct StateMachine {
    state: Arc<Mutex<State>>,
    sender: mpsc::Sender<ConnectionEvent>,
}

impl StateMachine {
    pub fn new(sender: mpsc::Sender<ConnectionEvent>) -> Self {
        Self {
            state: Arc::new(Mutex::new(State::Idle)),
            sender,
        }
    }

    pub fn handle_event(&self, event: ConnectionEvent) {
        let mut state = self.state.lock().unwrap();
        match *state {
            State::Idle => match event {
                ConnectionEvent::Connect => {
                    println!("Transitioning from Idle to Connecting");
                    *state = State::Connecting;
                }
                _ => println!("Unhandled event in Idle state"),
            },
            State::Connecting => match event {
                ConnectionEvent::Success => {
                    println!("Transitioning from Connecting to Connected");
                    *state = State::Connected;
                }
                ConnectionEvent::Failure => {
                    println!("Transitioning from Connecting to Error");
                    *state = State::Error;
                }
                _ => println!("Unhandled event in Connecting state"),
            },
            State::Connected => match event {
                ConnectionEvent::Disconnect => {
                    println!("Transitioning from Connected to Idle");
                    *state = State::Idle;
                }
                _ => println!("Unhandled event in Connected state"),
            },
            State::Error => match event {
                ConnectionEvent::Retry => {
                    println!("Transitioning from Error to Connecting");
                    *state = State::Connecting;
                }
                _ => println!("Unhandled event in Error state"),
            },
        }
    }
}

#[derive(Debug)]
pub enum ConnectionEvent {
    Connect,
    Success,
    Failure,
    Disconnect,
    Retry,
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
