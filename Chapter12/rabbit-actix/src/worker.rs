use actix::{Actor, AsyncContext, Context, Handler, Message, StreamHandler, System};
use actix::fut::wrap_future;
use failure::Error;
use futures::Future;
use lapin::types::FieldTable;
use lapin::channel::{BasicConsumeOptions, Channel};
use lapin::consumer::Consumer;
use lapin::error::Error as LapinError;
use lapin::message::Delivery;
use log::debug;
use tokio::net::TcpStream;

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
    }
}

impl Actor for WorkerActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let chan = self.channel.clone();
        let addr = ctx.address();
        let fut = rabbit_actix::ensure_queue(&chan)
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

fn main() -> Result<(), Error> {
    env_logger::init();
    let mut sys = System::new("rabbit-actix-worker");

    let channel = rabbit_actix::spawn_client(&mut sys)?;
    let actor = WorkerActor { channel };
    let _addr = actor.start();

    sys.run();
    Ok(())
}
