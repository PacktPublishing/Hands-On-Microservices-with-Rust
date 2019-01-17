use actix::{Actor, AsyncContext, Context, Handler, Message, StreamHandler, System};
use actix::fut::wrap_future;
use image::GenericImageView;
use failure::{format_err, Error};
use futures::Future;
use lapin::types::FieldTable;
use lapin::channel::{BasicConsumeOptions, BasicProperties, BasicPublishOptions, Channel};
use lapin::consumer::Consumer;
use lapin::error::Error as LapinError;
use lapin::message::Delivery;
use log::{debug, warn};
use queens_rock::Scanner;
use rabbit_actix::{QrRequest, QrResponse, REQUESTS, RESPONSES};
use rabbit_actix::queue_actor::{QueueActor, QueueHandler, TaskId};
use tokio::net::TcpStream;

/*
struct AttachStream(Consumer<TcpStream>);

impl Message for AttachStream {
    type Result = ();
}

struct WorkerActor {
    channel: Channel<TcpStream>,
}

impl Handler<AttachStream> for WorkerActor {
    type Result = ();

    fn handle(&mut self, msg: AttachStream, ctx: &mut Self::Context) -> Self::Result {
        debug!("subscribed");
        ctx.add_stream(msg.0);
    }
}

impl StreamHandler<Delivery, LapinError> for WorkerActor {
    fn handle(&mut self, item: Delivery, ctx: &mut Context<Self>) {
        debug!("Message received!");
        let fut = self.channel
            .basic_ack(item.delivery_tag, false)
            .map_err(drop);
        ctx.spawn(wrap_future(fut));
        if let Some(corr_id) = item.properties.correlation_id() {
            let opts = BasicPublishOptions::default();
            let props = BasicProperties::default()
                .with_correlation_id(corr_id.to_owned());
            let data = "content".to_string().into_bytes();
            let fut = self.channel
                .basic_publish("", RESPONSES, data, opts, props)
                .map(drop)
                .map_err(drop);
            ctx.spawn(wrap_future(fut));
        } else {
            warn!("Message has no address for the response");
        }
    }
}

impl Actor for WorkerActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let chan = self.channel.clone();
        let addr = ctx.address();
        let fut = ensure_queue(&chan, REQUESTS)
            .and_then(move |queue| {
                let opts = BasicConsumeOptions::default();
                let table = FieldTable::new();
                chan.basic_consume(&queue, "worker", opts, table)
            })
            .from_err::<Error>()
            .and_then(move |stream| {
                debug!("Stream!");
                addr.send(AttachStream(stream))
                    .from_err::<Error>()
            })
            .map(drop)
            .map_err(drop);
        ctx.spawn(wrap_future(fut));
    }
}
*/

struct WokerHandler {
}

impl WokerHandler {
    fn scan(&self, data: &[u8]) -> Result<String, Error> {
        let image = image::load_from_memory(data)?;
        let luma = image.to_luma().into_vec();
        let scanner = Scanner::new(
            luma.as_ref(),
            image.width() as usize,
            image.height() as usize,
            );
        scanner
            .scan()
            .extract(0)
            .ok_or_else(|| format_err!("can't extract"))
            .and_then(|code| {
                code.decode()
                    .map_err(|_| format_err!("can't decode"))
            })
            .and_then(|data| {
                data.try_string()
                    .map_err(|_| format_err!("can't convert to a string"))
            })
    }
}

impl QueueHandler for WokerHandler {
    type Incoming = QrRequest;
    type Outgoing = QrResponse;

    fn incoming(&self) -> &str {
        REQUESTS
    }
    fn outgoing(&self) -> &str {
        RESPONSES
    }
    fn handle(&self, _: &TaskId, incoming: Self::Incoming)
        -> Result<Option<Self::Outgoing>, Error>
    {
        debug!("In: {:?}", incoming);
        let outgoing = self.scan(&incoming.image).into();
        debug!("Out: {:?}", outgoing);
        Ok(Some(outgoing))
    }
}

fn main() -> Result<(), Error> {
    env_logger::init();
    let mut sys = System::new("rabbit-actix-worker");

    //let channel = rabbit_actix::spawn_client(&mut sys)?;
    let addr = QueueActor::new(WokerHandler {}, &mut sys)?;
    //let _addr = actor.start();

    let _ = sys.run();
    Ok(())
}
