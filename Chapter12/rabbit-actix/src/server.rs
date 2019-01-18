use actix::{Addr, System};
use actix_web::dev::Payload;
use actix_web::error::MultipartError;
use actix_web::http::{self, header, StatusCode};
use actix_web::multipart::MultipartItem;
use actix_web::{
    middleware, server, App, Error as WebError, HttpMessage, HttpRequest, HttpResponse,
};
use askama::Template;
use chrono::{DateTime, Utc};
use failure::Error;
use futures::{future, Future, Stream};
use indexmap::IndexMap;
use log::debug;
use rabbit_actix::queue_actor::{QueueActor, QueueHandler, SendMessage, TaskId};
use rabbit_actix::{QrRequest, QrResponse, REQUESTS, RESPONSES};
use std::fmt;
use std::sync::{Arc, Mutex};

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
    Done(QrResponse),
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Status::InProgress => write!(f, "in progress"),
            Status::Done(resp) => match resp {
                QrResponse::Succeed(data) => write!(f, "done: {}", data),
                QrResponse::Failed(err) => write!(f, "failed: {}", err),
            },
        }
    }
}

type SharedTasks = Arc<Mutex<IndexMap<String, Record>>>;

#[derive(Clone)]
struct State {
    tasks: SharedTasks,
    addr: Addr<QueueActor<ServerHandler>>,
}

pub fn handle_multipart_item(
    item: MultipartItem<Payload>,
) -> Box<Stream<Item = Vec<u8>, Error = MultipartError>> {
    match item {
        MultipartItem::Field(field) => {
            Box::new(field.concat2().map(|bytes| bytes.to_vec()).into_stream())
        }
        MultipartItem::Nested(mp) => Box::new(mp.map(handle_multipart_item).flatten()),
    }
}

fn upload_handler(req: HttpRequest<State>) -> impl Future<Item = HttpResponse, Error = WebError> {
    req.multipart()
        .map(handle_multipart_item)
        .flatten()
        .into_future()
        .and_then(|(bytes, stream)| {
            if let Some(bytes) = bytes {
                Ok(bytes)
            } else {
                Err((MultipartError::Incomplete, stream))
            }
        })
        .map_err(|(err, _)| WebError::from(err))
        .and_then(move |image| {
            debug!("Image: {:?}", image);
            let request = QrRequest { image };
            req.state()
                .addr
                .send(SendMessage(request))
                .from_err()
                .map(move |task_id| {
                    let record = Record {
                        task_id: task_id.clone(),
                        timestamp: Utc::now(),
                        status: Status::InProgress,
                    };
                    req.state().tasks.lock().unwrap().insert(task_id, record);
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

fn tasks_handler(req: HttpRequest<State>) -> impl Future<Item = HttpResponse, Error = WebError> {
    let tasks: Vec<_> = req
        .state()
        .tasks
        .lock()
        .unwrap()
        .values()
        .cloned()
        .collect();
    let tmpl = Tasks { tasks };
    future::ok(HttpResponse::Ok().body(tmpl.render().unwrap()))
}

fn index_handler(_: &HttpRequest<State>) -> HttpResponse {
    HttpResponse::Ok().body("QR Parsing Microservice")
}

fn main() -> Result<(), Error> {
    env_logger::init();
    let mut sys = System::new("rabbit-actix-server");
    let tasks = Arc::new(Mutex::new(IndexMap::new()));
    let addr = QueueActor::new(
        ServerHandler {
            tasks: tasks.clone(),
        },
        &mut sys,
    )?;

    let state = State {
        tasks: tasks.clone(),
        addr,
    };
    server::new(move || {
        App::with_state(state.clone())
            .middleware(middleware::Logger::default())
            .resource("/", |r| r.f(index_handler))
            .resource("/task", |r| {
                //r.method(http::Method::GET).with_async(snd_msg);
                r.method(http::Method::POST).with_async(upload_handler);
            })
            .resource("/tasks", |r| r.method(http::Method::GET).with_async(tasks_handler))
    })
    .bind("127.0.0.1:8080")
    .unwrap()
    .start();

    let _ = sys.run();
    Ok(())
}

struct ServerHandler {
    tasks: SharedTasks,
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
    fn handle(
        &self,
        id: &TaskId,
        incoming: Self::Incoming,
    ) -> Result<Option<Self::Outgoing>, Error> {
        debug!("Result returned: {:?}", incoming);
        self.tasks.lock().unwrap().get_mut(id).map(move |rec| {
            rec.status = Status::Done(incoming);
        });
        Ok(None)
    }
}
