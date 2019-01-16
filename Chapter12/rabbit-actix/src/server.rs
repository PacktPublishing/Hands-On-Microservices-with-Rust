use actix::{Actor, Addr, System};
use actix_web::{http, middleware, server, App, Error as WebError, HttpRequest, HttpResponse};
use failure::Error;
use futures::Future;
use lapin::channel::{BasicProperties, BasicPublishOptions, Channel};
use log::debug;
use rabbit_actix::{REQUESTS, RESPONSES};
use rabbit_actix::queue_actor::{SendMessage, QueueActor, QueueHandler};
use std::cell::RefCell;
use tokio::net::TcpStream;

#[derive(Clone)]
struct State {
    cell: RefCell<u8>,
    channel: Channel<TcpStream>,
    addr: Addr<QueueActor<ServerHandler>>,
}

fn snd_msg(req: HttpRequest<State>)
    -> impl Future<Item = HttpResponse, Error = WebError>
{
    let inc = *req.state().cell.borrow() + 1;
    req.state().cell.replace(inc);
    req.state().addr.send(SendMessage(format!("value={}", inc)))
        .from_err::<WebError>()
        .map(|_| {
            HttpResponse::Ok().body(format!("Sent"))
        })
}

/*
fn index(req: HttpRequest<State>)
    -> impl Future<Item = HttpResponse, Error = WebError>
{
    let opts = BasicPublishOptions::default();
    let prop = BasicProperties::default()
        .with_delivery_mode(2)
        .with_correlation_id("corr".to_string());
    let data = "content".to_string().into_bytes();
    req.state().channel.basic_publish("", rabbit_actix::REQUESTS, data, opts, prop)
        .map_err(|err| WebError::from(Error::from(err)))
        .map(|_| {
            HttpResponse::Ok().body(format!("Sent"))
        })
}
*/

fn main() -> Result<(), Error> {
    env_logger::init();
    let mut sys = System::new("rabbit-actix-server");
    let channel = rabbit_actix::spawn_client(&mut sys)?;
    let actor = QueueActor::new(channel.clone(), ServerHandler {});
    let addr = actor.start();

    let state = State { cell: RefCell::new(0), channel, addr };
    server::new(move || {
        App::with_state(state.clone())
            .middleware(middleware::Logger::default())
            //.resource("/", |r| r.f(index))
            .resource("/", |r| r.method(http::Method::GET).with_async(snd_msg))
            //.resource("/x", |r| r.method(http::Method::GET).with_async(index))
    }).bind("127.0.0.1:8080")
    .unwrap()
        .start();

    sys.run();
    Ok(())
}

struct ServerHandler {
}

impl QueueHandler for ServerHandler {
    type Incoming = String;
    type Outgoing = String;

    fn incoming(&self) -> &str {
        RESPONSES
    }
    fn outgoing(&self) -> &str {
        REQUESTS
    }
    fn handle(&self, incoming: Self::Incoming) -> Result<Option<Self::Outgoing>, Error> {
        debug!("RESULT RETURNED! :{}", incoming);
        Ok(None)
    }
}

