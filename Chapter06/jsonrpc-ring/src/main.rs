use failure::Error;
use jsonrpc::client::Client;
use jsonrpc::error::Error as ClientError;
use jsonrpc::simple_http::Builder;
use jsonrpc::serde_json::value::RawValue;
use jsonrpc_http_server::ServerBuilder;
use jsonrpc_http_server::jsonrpc_core::{IoHandler, Error as ServerError, Value};
use log::{debug, error, trace};
use serde::Deserialize;
use std::env;
use std::fmt;
use std::net::SocketAddr;
use std::sync::Mutex;
use std::sync::mpsc::{channel, Sender};
use std::thread;

const START_ROLL_CALL: &str = "start_roll_call";
const MARK_ITSELF: &str = "mark_itself";

enum Action {
    StartRollCall,
    MarkItself,
}

struct Remote {
    client: Client,
}

impl Remote {
    fn new(addr: SocketAddr) -> Self {
        let url = format!("http://{}", addr);
        let transport = Builder::new()
            .url(&url)
            .unwrap()
            .build();
        let client = Client::with_transport(transport);
        Self {
            client
        }
    }

    fn start_roll_call(&self) -> Result<bool, ClientError> {
        self.call_method(START_ROLL_CALL, &[])
    }

    fn mark_itself(&self) -> Result<bool, ClientError> {
        self.call_method(MARK_ITSELF, &[])
    }

    fn call_method<T>(&self, meth: &str, args: &[Box<RawValue>]) -> Result<T, ClientError>
    where
        T: for <'de> Deserialize<'de>,
    {
        let request = self.client.build_request(meth, args);
        self.client.send_request(request).and_then(|res| res.result::<T>())
    }
}

fn to_internal<E: fmt::Display>(err: E) -> ServerError {
    error!("Error: {}", err);
    ServerError::internal_error()
}

fn spawn_worker() -> Result<Sender<Action>, Error> {
    let (tx, rx) = channel();
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
    Ok(tx)
}

fn main() -> Result<(), Error> {
    env_logger::init();
    let tx = spawn_worker()?;
    let addr: SocketAddr = env::var("ADDRESS")?.parse()?;
    let mut io = IoHandler::default();
    let sender = Mutex::new(tx.clone());
    io.add_sync_method(START_ROLL_CALL, move |_| {
        trace!("START_ROLL_CALL");
        let tx = sender
            .lock()
            .map_err(to_internal)?;
        tx.send(Action::StartRollCall)
            .map_err(to_internal)
            .map(|_| Value::Bool(true))
    });
    let sender = Mutex::new(tx.clone());
    io.add_sync_method(MARK_ITSELF, move |_| {
        trace!("MARK_ITSELF");
        let tx = sender
            .lock()
            .map_err(to_internal)?;
        tx.send(Action::MarkItself)
            .map_err(to_internal)
            .map(|_| Value::Bool(true))
    });


    let server = ServerBuilder::new(io).start_http(&addr)?;

    Ok(server.wait())
}
