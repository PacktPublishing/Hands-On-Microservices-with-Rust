use actix::{Actor, AsyncContext, Context, Handler, Message, StreamHandler};
use actix::fut::wrap_future;
use failure::{format_err, Error};
use futures::Future;
use lapin::types::{FieldTable, ShortString};
use lapin::channel::{BasicConsumeOptions, BasicProperties, BasicPublishOptions, Channel};
use lapin::consumer::Consumer;
use lapin::error::Error as LapinError;
use lapin::message::Delivery;
use log::{debug, warn};
use serde::{Deserialize, Serialize};
use super::ensure_queue;
use tokio::net::TcpStream;
use uuid::Uuid;

pub trait QueueHandler: 'static {
    type Incoming: for <'de> Deserialize<'de>;
    type Outgoing: Serialize;

    fn incoming(&self) -> &str;
    fn outgoing(&self) -> &str;
    fn handle(&self, incoming: Self::Incoming) -> Result<Option<Self::Outgoing>, Error>;
}

pub type TaskId = ShortString;

struct AttachStream(Consumer<TcpStream>);

impl Message for AttachStream {
    type Result = ();
}

pub struct SendMessage<T>(pub T);

impl<T> Message for SendMessage<T> {
    type Result = TaskId;
}

pub struct QueueActor<T: QueueHandler> {
    channel: Channel<TcpStream>,
    handler: T,
}

impl<T: QueueHandler> QueueActor<T> {
    pub fn new(channel: Channel<TcpStream>, handler: T) -> Self {
        Self { channel, handler }
    }
}

impl<T: QueueHandler> Handler<AttachStream> for QueueActor<T> {
    type Result = ();

    fn handle(&mut self, msg: AttachStream, ctx: &mut Self::Context) -> Self::Result {
        debug!("subscribed");
        ctx.add_stream(msg.0);
    }
}

impl<T: QueueHandler> Handler<SendMessage<T::Outgoing>> for QueueActor<T> {
    type Result = TaskId;

    fn handle(&mut self, msg: SendMessage<T::Outgoing>, ctx: &mut Self::Context) -> Self::Result {
        let corr_id = Uuid::new_v4().to_simple().to_string();
        self.send_message(corr_id.clone(), msg.0, ctx);
        corr_id
    }
}

impl<T: QueueHandler> StreamHandler<Delivery, LapinError> for QueueActor<T> {
    fn handle(&mut self, item: Delivery, ctx: &mut Context<Self>) {
        debug!("Message received!");
        let fut = self.channel
            .basic_ack(item.delivery_tag, false)
            .map_err(drop);
        ctx.spawn(wrap_future(fut));
        match self.process_message(item, ctx) {
            Ok(pair) => {
                if let Some((corr_id, data)) = pair {
                    self.send_message(corr_id, data, ctx);
                }
            }
            Err(err) => {
                warn!("Message processing error: {}", err);
            }
        }
    }
}

impl<T: QueueHandler> QueueActor<T> {
    fn process_message(&self, item: Delivery, ctx: &mut Context<Self>)
        -> Result<Option<(ShortString, T::Outgoing)>, Error>
    {
        debug!("- - - Received!");
        let incoming = rmp_serde::decode::from_slice(&item.data)?;
        let outgoing = self.handler.handle(incoming)?;
        if let Some(outgoing) = outgoing {
            let corr_id = item.properties.correlation_id()
                .to_owned()
                .ok_or_else(|| format_err!("Message has no address for the response"))?;
            Ok(Some((corr_id, outgoing)))
        } else {
            Ok(None)
        }
    }

    fn send_message(&self, corr_id: ShortString, outgoing: T::Outgoing, ctx: &mut Context<Self>) {
        debug!("- - - Sending!");
        let data = rmp_serde::encode::to_vec(&outgoing);
        match data {
            Ok(data) => {
                let opts = BasicPublishOptions::default();
                let props = BasicProperties::default()
                    //.with_delivery_mode(2)
                    .with_correlation_id(corr_id);
                debug!("Sending to: {}", self.handler.outgoing());
                let fut = self.channel
                    .basic_publish("", self.handler.outgoing(), data, opts, props)
                    .map(drop)
                    .map_err(drop);
                ctx.spawn(wrap_future(fut));
            }
            Err(err) => {
                warn!("Can't encode an outgoing message: {}", err);
            }
        }
    }
}

impl<T: QueueHandler> Actor for QueueActor<T> {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let chan = self.channel.clone();
        let fut = ensure_queue(&chan, self.handler.outgoing())
            .map(drop)
            .map_err(drop);
        ctx.spawn(wrap_future(fut));
        let addr = ctx.address();
        let fut = ensure_queue(&chan, self.handler.incoming())
            .and_then(move |queue| {
                let opts = BasicConsumeOptions {
                    ..Default::default()
                };
                let table = FieldTable::new();
                debug!("Receiving from: {}", queue.name());
                chan.basic_consume(&queue, "consumer", opts, table)
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
