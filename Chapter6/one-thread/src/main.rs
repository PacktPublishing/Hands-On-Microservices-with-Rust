#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate rand;
extern crate futures;
extern crate tokio;
extern crate hyper;
extern crate serde_json;
extern crate hyper_staticfile;
extern crate image;
extern crate queryst;

use std::io::{Error, ErrorKind};
use std::fs;
use std::thread;
use std::path::Path;
use regex::Regex;
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use futures::{future, Future, Sink, Stream};
use futures::sync::{mpsc, oneshot};
use serde_json::Value;
use tokio::fs::File;
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use hyper::service::service_fn;
use hyper_staticfile::FileChunkStream;

static INDEX: &[u8] = b"Resize Microservice";

struct WorkerRequest {
    buffer: Vec<u8>,
    width: u16,
    height: u16,
    tx: oneshot::Sender<Result<WorkerResponse, Error>>,
}

struct WorkerResponse {
    buffer: Vec<u8>,
}

fn start_worker() -> mpsc::Sender<WorkerRequest> {
    let (tx, rx) = mpsc::channel(1);
    thread::spawn(move || {
        let requests = rx.wait();
        for req in requests {
        }
    });
    tx
}

fn convert(data: Vec<u8>, width: u16, height: u16) -> Result<Vec<u8>, Error> {
    Err(other("not implemented"))
}

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

fn microservice_handler(tx: mpsc::Sender<WorkerRequest>, req: Request<Body>)
    -> Box<Future<Item=Response<Body>, Error=Error> + Send>
{
    match (req.method(), req.uri().path().to_owned().as_ref()) {
        (&Method::GET, "/") => {
            Box::new(future::ok(Response::new(INDEX.into())))
        },
        (&Method::POST, "/resize") => {
            let (width, height) = {
                let uri = req.uri().query().unwrap_or("");
                let query = queryst::parse(uri).unwrap_or(Value::Null);
                let w = to_number(&query["width"], 180);
                let h = to_number(&query["height"], 180);
                (w, h)
            };
            let body = req.into_body()
                .map_err(other)
                .concat2()
                .map(|chunk| {
                    chunk.to_vec()
                })
                .and_then(move |buffer| {
                    let (resp_tx, resp_rx) = oneshot::channel();
                    let resp_rx = resp_rx.map_err(other);
                    let request = WorkerRequest {
                        buffer,
                        width,
                        height,
                        tx: resp_tx,
                    };
                    tx.send(request)
                        .map_err(other)
                        .and_then(move |_| resp_rx)
                        .and_then(|x| x)
                })
                .map(|resp| {
                    Response::new(resp.buffer.into())
                });
            Box::new(body)
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


fn main() {
    let addr = ([127, 0, 0, 1], 8080).into();
    let builder = Server::bind(&addr);
    let tx = start_worker();
    let server = builder.serve(move || {
        let tx = tx.clone();
        service_fn(move |req| microservice_handler(tx.clone(), req))
    });
    let server = server.map_err(drop);
    hyper::rt::run(server);
}
