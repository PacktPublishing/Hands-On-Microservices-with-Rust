use super::ensure_queue;
use actix::fut::wrap_future;
use actix::{Actor, Addr, AsyncContext, Context, Handler, Message, StreamHandler, SystemRunner};
use failure::{format_err, Error};
use futures::Future;
use lapin::channel::{BasicConsumeOptions, BasicProperties, BasicPublishOptions, Channel};
use lapin::consumer::Consumer;
use lapin::error::Error as LapinError;
use lapin::message::Delivery;
use lapin::types::{FieldTable, ShortString};
use log::{debug, warn};
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use uuid::Uuid;

pub trait QueueHandler: 'static {
    type Incoming: for<'de> Deserialize<'de>;
    type Outgoing: Serialize;

    fn incoming(&self) -> &str;
    fn outgoing(&self) -> &str;
    fn handle(
        &self,
        id: &TaskId,
        incoming: Self::Incoming,
    ) -> Result<Option<Self::Outgoing>, Error>;
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
    pub fn new(handler: T, mut sys: &mut SystemRunner) -> Result<Addr<Self>, Error> {
        let channel = super::spawn_client(&mut sys)?;
        let chan = channel.clone();
        let fut = ensure_queue(&chan, handler.outgoing());
        sys.block_on(fut)?;
        let fut = ensure_queue(&chan, handler.incoming()).and_then(move |queue| {
            let opts = BasicConsumeOptions {
                ..Default::default()
            };
            let table = FieldTable::new();
            let name = format!("{}-consumer", queue.name());
            chan.basic_consume(&queue, &name, opts, table)
        });
        let stream = sys.block_on(fut)?;
        let addr = QueueActor::create(move |ctx| {
            ctx.add_stream(stream);
            Self { channel, handler }
        });
        Ok(addr)
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
        let fut = self
            .channel
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
    fn process_message(
        &self,
        item: Delivery,
        ctx: &mut Context<Self>,
    ) -> Result<Option<(ShortString, T::Outgoing)>, Error> {
        debug!("- - - Received!");
        let corr_id = item
            .properties
            .correlation_id()
            .to_owned()
            .ok_or_else(|| format_err!("Message has no address for the response"))?;
        let incoming = serde_json::from_slice(&item.data)?;
        let outgoing = self.handler.handle(&corr_id, incoming)?;
        if let Some(outgoing) = outgoing {
            Ok(Some((corr_id, outgoing)))
        } else {
            Ok(None)
        }
    }

    fn send_message(&self, corr_id: ShortString, outgoing: T::Outgoing, ctx: &mut Context<Self>) {
        debug!("- - - Sending!");
        let data = serde_json::to_vec(&outgoing);
        match data {
            Ok(data) => {
                let opts = BasicPublishOptions::default();
                let props = BasicProperties::default().with_correlation_id(corr_id);
                debug!("Sending to: {}", self.handler.outgoing());
                let fut = self
                    .channel
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

    fn started(&mut self, ctx: &mut Self::Context) {}
}
