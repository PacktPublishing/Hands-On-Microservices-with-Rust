extern crate futures;
extern crate hyper;
extern crate serde_json;
extern crate image;
extern crate queryst;
extern crate futures_cpupool;

use image::{ImageResult, FilterType};
use std::io::{Error, ErrorKind};
use futures::{future, Future, Stream};
use serde_json::Value;
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use hyper::service::service_fn;
use futures_cpupool::CpuPool;

static INDEX: &[u8] = b"Resize Microservice";

fn convert(data: &[u8], width: u16, height: u16) -> ImageResult<Vec<u8>> {
    let format = image::guess_format(&data)?;
    let img = image::load_from_memory(&data)?;
    let scaled = img.resize(width as u32, height as u32, FilterType::Lanczos3);
    let mut result = Vec::new();
    scaled.write_to(&mut result, format)?;
    Ok(result)
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

fn microservice_handler(pool: CpuPool, req: Request<Body>)
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
                    let task = future::lazy(move || {
                        convert(&buffer, width, height)
                    });
                    pool.spawn(task)
                        .map_err(other)
                })
                .map(|resp| {
                    Response::new(resp.into())
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
    let pool = CpuPool::new(4);
    let builder = Server::bind(&addr);
    let server = builder.serve(move || {
        let pool = pool.clone();
        service_fn(move |req| microservice_handler(pool.clone(), req))
    });
    let server = server.map_err(drop);
    hyper::rt::run(server);
}
