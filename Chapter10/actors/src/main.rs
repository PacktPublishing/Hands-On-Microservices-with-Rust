use actix::{Actor, Addr};
use actix::sync::SyncArbiter;
use std::io::{Error, ErrorKind};
use failure::Fail;
use futures::{future, Future, Stream};
use serde_json::Value;
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use hyper::service::service_fn;

mod actors;

use self::actors::{
    count::{Count, CountActor},
    log::{Log, LogActor},
    resize::{Resize, ResizeActor},
};

static INDEX: &[u8] = b"Resize Microservice";

fn other<E>(err: E) -> Error
where
    E: Into<Box<std::error::Error + Send + Sync>>,
{
    Error::new(ErrorKind::Other, err)
}

fn to_number(value: &Value, default: u16) -> u16 {
    value.as_str()
        .and_then(|x| x.parse::<u16>().ok())
        .unwrap_or(default)
}

fn count_up(state: &State, path: &str) -> impl Future<Item=(), Error=Error> {
    let path = path.to_string();
    let log = state.log.clone();
    state.count.send(Count(path.clone()))
        .and_then(move |value| {
            let message = format!("total requests for '{}' is {}", path, value);
            log.send(Log(message))
        })
        .map_err(|err| other(err.compat()))
}

fn microservice_handler(state: &State, req: Request<Body>)
    -> Box<Future<Item=Response<Body>, Error=Error> + Send>
{
    match (req.method(), req.uri().path().to_owned().as_ref()) {
        (&Method::GET, "/") => {
            let fut = count_up(state, "/").map(|_| Response::new(INDEX.into()));
            Box::new(fut)
        },
        (&Method::POST, "/resize") => {
            let (width, height) = {
                let uri = req.uri().query().unwrap_or("");
                let query = queryst::parse(uri).unwrap_or(Value::Null);
                let w = to_number(&query["width"], 180);
                let h = to_number(&query["height"], 180);
                (w, h)
            };
            let resize = state.resize.clone();
            let body = req.into_body()
                .map_err(other)
                .concat2()
                .map(|chunk| {
                    chunk.to_vec()
                })
                .and_then(move |buffer| {
                    let msg = Resize {
                        buffer,
                        width,
                        height,
                    };
                    resize.send(msg)
                        .map_err(|err| other(err.compat()))
                        .and_then(|x| x.map_err(other))
                })
                .map(|resp| {
                    Response::new(resp.into())
                });
            let fut = count_up(state, "/resize").and_then(move |_| body);
            Box::new(fut)
        },
        _ => {
            response_with_code(StatusCode::NOT_FOUND)
        },
    }
}

fn response_with_code(status_code: StatusCode)
    -> Box<Future<Item=Response<Body>, Error=Error> + Send>
{
    let resp = Response::builder()
        .status(status_code)
        .body(Body::empty())
        .unwrap();
    Box::new(future::ok(resp))
}

#[derive(Clone)]
struct State {
    resize: Addr<ResizeActor>,
    count: Addr<CountActor>,
    log: Addr<LogActor>,
}

fn main() {
    actix::run(|| {
        let resize = SyncArbiter::start(2, || ResizeActor);
        let count = CountActor::new().start();
        let log = LogActor::new().start();

        let state = State { resize, count, log };

        let addr = ([127, 0, 0, 1], 8080).into();
        let builder = Server::bind(&addr);
        let server = builder.serve(move || {
            let state = state.clone();
            service_fn(move |req| microservice_handler(&state, req))
        });
        server.map_err(drop)
    });
}
