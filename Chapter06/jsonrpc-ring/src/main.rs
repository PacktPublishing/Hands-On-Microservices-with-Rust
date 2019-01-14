use failure::Error;
use jsonrpc::client::Client;
use jsonrpc::error::Error as ClientError;
use jsonrpc_http_server::ServerBuilder;
use jsonrpc_http_server::jsonrpc_core::{IoHandler, Error as ServerError, Value};
use log::{debug, error, trace};
use serde::Deserialize;
use std::env;
use std::fmt;
use std::net::SocketAddr;
use std::sync::{mpsc, Mutex};
use std::thread;

const START_ROLL_CALL: &str = "start_roll_call";
const MARK_ITSELF: &str = "mark_itself";

enum Action {
    StartRollCall,
    MarkItself,
}

struct Remote {
    addr: String,
}

impl Remote {
    fn new(addr: SocketAddr) -> Self {
        Self {
            addr: format!("http://{}", addr),
        }
    }

    fn start_roll_call(&self) -> Result<bool, ClientError> {
        self.call_method("start_roll_call", &[])
    }

    fn mark_itself(&self) -> Result<bool, ClientError> {
        self.call_method("mark_itself", &[])
    }

    fn call_method<T>(&self, meth: &str, args: &[Value]) -> Result<T, ClientError>
    where
        T: for <'de> Deserialize<'de>,
    {
        let client = Client::new(self.addr.clone(), None, None);
        let request = client.build_request(meth, args);
        client.send_request(&request).and_then(|res| res.into_result::<T>())
    }
}

fn to_internal<E: fmt::Display>(err: E) -> ServerError {
    error!("Error: {}", err);
    ServerError::internal_error()
}

fn main() -> Result<(), Error> {
    env_logger::init();

    let addr: SocketAddr = env::var("ADDRESS")?.parse()?;
    let (tx, rx) = mpsc::channel();

    let mut io = IoHandler::default();
    let sender = Mutex::new(tx.clone());
    io.add_method(START_ROLL_CALL, move |_| {
        trace!("START_ROLL_CALL");
        let tx = sender
            .lock()
            .map_err(to_internal)?;
        tx.send(Action::StartRollCall)
            .map_err(to_internal)
            .map(|_| Value::Bool(true))
    });
    let sender = Mutex::new(tx.clone());
    io.add_method(MARK_ITSELF, move |_| {
        trace!("MARK_ITSELF");
        let tx = sender
            .lock()
            .map_err(to_internal)?;
        tx.send(Action::MarkItself)
            .map_err(to_internal)
            .map(|_| Value::Bool(true))
    });

    let next: SocketAddr = env::var("NEXT")?.parse()?;
    thread::spawn(move || {
        let remote = Remote::new(next);
        let mut in_roll_call = false;
        for action in rx.iter() {
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
    });

    let server = ServerBuilder::new(io).start_http(&addr)?;

    Ok(server.wait())
}
