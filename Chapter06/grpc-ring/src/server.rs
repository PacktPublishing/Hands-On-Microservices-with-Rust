mod ring;
mod ring_grpc;

use crate::ring::Empty;
use crate::ring_grpc::{Ring, RingServer};
use failure::Error;
use grpc::{Error as GrpcError, ServerBuilder, SingleResponse, RequestOptions};
use grpc_ring::Remote;
use log::{debug, trace};
use std::env;
use std::net::SocketAddr;
use std::sync::Mutex;
use std::sync::mpsc::{channel, Receiver, Sender};

macro_rules! try_or_response {
    ($x:expr) => {{
        match $x {
            Ok(value) => {
                value
            }
            Err(err) => {
                let error = GrpcError::Panic(err.to_string());
                return SingleResponse::err(error);
            }
        }
    }};
}

enum Action {
    StartRollCall,
    MarkItself,
}

struct RingImpl {
    sender: Mutex<Sender<Action>>,
}

impl RingImpl {
    fn new(sender: Sender<Action>) -> Self {
        Self {
            sender: Mutex::new(sender),
        }
    }

    fn send_action(&self, action: Action) -> SingleResponse<Empty> {
        let tx = try_or_response!(self.sender.lock());
        try_or_response!(tx.send(action));
        let result = Empty::new();
        SingleResponse::completed(result)
    }
}

impl Ring for RingImpl {
    fn start_roll_call(&self, _: RequestOptions, _: Empty) -> SingleResponse<Empty> {
        trace!("START_ROLL_CALL");
        self.send_action(Action::StartRollCall)
    }

    fn mark_itself(&self, _: RequestOptions, _: Empty) -> SingleResponse<Empty> {
        trace!("MARK_ITSELF");
        self.send_action(Action::MarkItself)
    }
}

fn main() -> Result<(), Error> {
    env_logger::init();
    let (tx, rx) = channel();
    let addr: SocketAddr = env::var("ADDRESS")?.parse()?;
    let mut server = ServerBuilder::new_plain();
    server.http.set_addr(addr)?;
    let ring = RingImpl::new(tx);
    server.add_service(RingServer::new_service_def(ring));
    server.http.set_cpu_pool_threads(4);
    let _server = server.build()?;

    worker_loop(rx)
}

fn worker_loop(receiver: Receiver<Action>) -> Result<(), Error> {
    let next = env::var("NEXT")?.parse()?;
    let remote = Remote::new(next)?;
    let mut in_roll_call = false;
    for action in receiver.iter() {
        match action {
            Action::StartRollCall => {
                if !in_roll_call {
                    if remote.start_roll_call().is_ok() {
                        debug!("ON");
                        in_roll_call = true;
                    }
                } else {
                    if remote.mark_itself().is_ok() {
                        debug!("OFF");
                        in_roll_call = false;
                    }
                }
            }
            Action::MarkItself => {
                if in_roll_call {
                    if remote.mark_itself().is_ok() {
                        debug!("OFF");
                        in_roll_call = false;
                    }
                } else {
                    debug!("SKIP");
                }
            }
        }
    }
    Ok(())
}
