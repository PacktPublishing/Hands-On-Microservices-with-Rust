use actix::{Actor, Addr, System};
use actix_web::{middleware, server, App, Error as WebError, HttpMessage, HttpRequest, HttpResponse};
use actix_web::error::MultipartError;
use actix_web::http::{self, header, StatusCode};
use actix_web::multipart::MultipartItem;
use askama::Template;
use chrono::{DateTime, Utc};
use indexmap::IndexMap;
use failure::Error;
use futures::{Future, Stream};
use futures::future::IntoFuture;
use lapin::channel::Channel;
use log::{debug, warn};
use rabbit_actix::{QrRequest, QrResponse, REQUESTS, RESPONSES};
use rabbit_actix::queue_actor::{SendMessage, TaskId, QueueActor, QueueHandler};
use std::fmt;
use std::sync::{Arc, Mutex};
use tokio::net::TcpStream;

#[derive(Template)]
#[template(path = "tasks.html")]
struct Tasks {
    tasks: Vec<Record>,
}

#[derive(Clone)]
struct Record {
    task_id: TaskId,
    timestamp: DateTime<Utc>,
    status: Status,
}

#[derive(Clone)]
enum Status {
    InProgress,
    Done,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = {
            match self {
                Status::InProgress => "in progress",
                Status::Done => "done",
            }
        };
        f.write_str(s)
    }
}

#[derive(Clone)]
struct State {
    tasks: Arc<Mutex<IndexMap<String, Record>>>,
    channel: Channel<TcpStream>,
    addr: Addr<QueueActor<ServerHandler>>,
}

fn boxed<I, E, F>(fut: F) -> Box<Future<Item = I, Error = E>>
where
    F: Future<Item = I, Error = E> + 'static,
{
    Box::new(fut)
}

fn upload(req: HttpRequest<State>)
    -> impl Future<Item = HttpResponse, Error = WebError>
{
    req.multipart()
        .into_future()
        .map_err(|(err, _)| err)
        .and_then(|(opt_item, _)| {
            if let Some(MultipartItem::Field(field)) = opt_item {
                debug!("Field: {:?}", field);
                boxed(field.concat2())
            } else {
                warn!("Unsupported multipart format");
                boxed(Err(MultipartError::Incomplete).into_future())
            }
        })
        .from_err()
        .and_then(move |bytes| {
            let request = QrRequest {
                image: bytes.to_vec(),
            };
            req.state().addr.send(SendMessage(request))
                .from_err()
                .map(move |task_id| {
                    let record = Record {
                        task_id: task_id.clone(),
                        timestamp: Utc::now(),
                        status: Status::InProgress,
                    };
                    req.state().tasks.lock()
                        .unwrap()
                        .insert(task_id, record);
                    req
                })
        })
        .map(|req| {
            HttpResponse::build_from(&req)
                .status(StatusCode::FOUND)
                .header(header::LOCATION, "/tasks")
                .finish()
        })
}

/*
fn snd_msg(req: HttpRequest<State>)
    -> impl Future<Item = HttpResponse, Error = WebError>
{
    req.state().addr.send(SendMessage(format!("value text")))
        .from_err::<WebError>()
        .map(move |task_id| {
            let record = Record {
                task_id: task_id.clone(),
                timestamp: Utc::now(),
                status: Status::InProgress,
            };
            req.state().tasks.lock()
                .unwrap()
                .insert(task_id, record);
            HttpResponse::build_from(&req)
                .status(StatusCode::FOUND)
                .header(header::LOCATION, "/tasks")
                .finish()
        })
}
*/

fn index(req: HttpRequest<State>)
    -> impl Future<Item = HttpResponse, Error = WebError>
{
    let tasks = req.state().tasks.lock()
        .unwrap()
        .values()
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
            .resource("/", |r| {
                //r.method(http::Method::GET).with_async(snd_msg);
                r.method(http::Method::POST).with_async(upload);
            })
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
    type Incoming = QrResponse;
    type Outgoing = QrRequest;

    fn incoming(&self) -> &str {
        RESPONSES
    }
    fn outgoing(&self) -> &str {
        REQUESTS
    }
    fn handle(&self, incoming: Self::Incoming) -> Result<Option<Self::Outgoing>, Error> {
        debug!("RESULT RETURNED! {:?}", incoming);
        Ok(None)
    }
}

