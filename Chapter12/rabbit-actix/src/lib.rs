pub mod queue_actor;

use actix::{Message, SystemRunner};
use failure::Error;
use futures::Future;
use lapin::client::{Client, ConnectionOptions};
use lapin::channel::{Channel, QueueDeclareOptions};
use lapin::error::Error as LapinError;
use lapin::queue::Queue;
use lapin::types::FieldTable;
use serde_derive::{Deserialize, Serialize};
use tokio::net::TcpStream;

pub const REQUESTS: &str = "requests";
pub const RESPONSES: &str = "responses";

pub fn spawn_client(sys: &mut SystemRunner) -> Result<Channel<TcpStream>, Error> {
    let addr = "127.0.0.1:5672".parse().unwrap();
    let fut = TcpStream::connect(&addr)
        .map_err(Error::from)
        .and_then(|stream| {
            let options = ConnectionOptions::default();
            Client::connect(stream, options)
                .from_err::<Error>()
        });
    let (client, heartbeat) = sys.block_on(fut)?;
    actix::spawn(heartbeat.map_err(drop));
    let channel = sys.block_on(client.create_channel())?;
    Ok(channel)
}

pub fn ensure_queue(chan: &Channel<TcpStream>, name: &str)
    -> impl Future<Item = Queue, Error = LapinError>
{
    let opts = QueueDeclareOptions {
        //durable: true,
        ..Default::default()
    };
    let table = FieldTable::new();
    chan.queue_declare(name, opts, table)
}

#[derive(Debug, Deserialize, Serialize)]
pub struct QrRequest {
    pub image: Vec<u8>,
}

impl Message for QrRequest {
    type Result = ();
}

#[derive(Debug, Deserialize, Serialize)]
pub struct QrResponse {
    pub link: String,
}

impl Message for QrResponse {
    type Result = ();
}
