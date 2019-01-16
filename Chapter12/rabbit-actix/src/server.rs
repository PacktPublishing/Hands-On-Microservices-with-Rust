use actix::{Actor, Addr, System};
use actix_web::{http, middleware, server, App, Error as WebError, HttpRequest, HttpResponse};
use askama::Template;
use indexmap::IndexMap;
use failure::Error;
use futures::Future;
use lapin::channel::Channel;
use log::debug;
use rabbit_actix::{REQUESTS, RESPONSES};
use rabbit_actix::queue_actor::{SendMessage, QueueActor, QueueHandler};
use std::sync::{Arc, Mutex};
use tokio::net::TcpStream;

#[derive(Template)]
#[template(path = "tasks.html")]
struct Tasks {
    tasks: Vec<String>,
}

enum Status {
    InProgress,
    Done,
}

#[derive(Clone)]
struct State {
    tasks: Arc<Mutex<IndexMap<String, Status>>>, // Add DateTime
    channel: Channel<TcpStream>,
    addr: Addr<QueueActor<ServerHandler>>,
}

fn snd_msg(req: HttpRequest<State>)
    -> impl Future<Item = HttpResponse, Error = WebError>
{
    req.state().addr.send(SendMessage(format!("value text")))
        .from_err::<WebError>()
        .map(move |task_id| {
            req.state().tasks.lock()
                .unwrap()
                .insert(task_id, Status::InProgress);
            HttpResponse::Ok().body(format!("Sent"))
        })
}

fn index(req: HttpRequest<State>)
    -> impl Future<Item = HttpResponse, Error = WebError>
{
    let tasks = req.state().tasks.lock()
        .unwrap()
        .keys()
        .cloned()
        .collect::<Vec<_>>();
        //.join(",");
    let tmpl = Tasks { tasks };
    futures::future::ok(HttpResponse::Ok().body(tmpl.render().unwrap()))
}

fn main() -> Result<(), Error> {
    env_logger::init();
    let mut sys = System::new("rabbit-actix-server");
    let channel = rabbit_actix::spawn_client(&mut sys)?;
    let actor = QueueActor::new(channel.clone(), ServerHandler {});
    let addr = actor.start();

    let state = State {
        tasks: Arc::new(Mutex::new(IndexMap::new())),
        channel,
        addr,
    };
    server::new(move || {
        App::with_state(state.clone())
            .middleware(middleware::Logger::default())
            //.resource("/", |r| r.f(index))
            .resource("/", |r| r.method(http::Method::GET).with_async(snd_msg))
            .resource("/tasks", |r| r.method(http::Method::GET).with_async(index))
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

