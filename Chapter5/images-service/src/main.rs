#[macro_use]
extern crate failure;
extern crate rand;
extern crate futures;
extern crate tokio;
extern crate hyper;

use std::io;
use std::fs;
use std::path::Path;
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use futures::{future, Future, Stream};
use tokio::fs::File;
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use hyper::service::service_fn;

static INDEX: &[u8] = b"Images Microservice";

#[derive(Debug, Fail)]
enum Error {
    #[fail(display = "server error: {}", _0)]
    HyperError(#[cause] hyper::Error),
    #[fail(display = "io error: {}", _0)]
    IoError(#[cause] io::Error),
}

impl From<hyper::Error> for Error {
    fn from(err: hyper::Error) -> Self {
        Error::HyperError(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::IoError(err)
    }
}

fn microservice_handler(req: Request<Body>, files: &Path)
    -> Box<Future<Item=Response<Body>, Error=String> + Send>
{
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => {
            Box::new(future::ok(Response::new(INDEX.into())))
        },
        (&Method::POST, "/upload") => {
            let name: String = thread_rng().sample_iter(&Alphanumeric).take(20).collect();
            let mut filepath = files.to_path_buf();
            filepath.push(&name);
            let create_file = File::create(filepath)
                .map_err(Error::from);
            let write = create_file.and_then(|file| {
                req.into_body()
                    .map_err(Error::from)
                    .fold(file, |file, chunk| {
                    tokio::io::write_all(file, chunk)
                        .map(|(file, _)| file)
                        .map_err(Error::from)
                })
            });
            let body = write.map(|_| {
                Response::new(name.into())
            }).map_err(|err| err.to_string());
            Box::new(body)
        },
        (&Method::GET, "/download") => {
            let body = Response::new("".into());
            Box::new(future::ok(body))
        },
        _ => {
            let resp = Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body("Not Found".into())
                .unwrap();
            Box::new(future::ok(resp))
        },
    }
}

fn main() {
    let files = Path::new("./files");
    fs::create_dir(files).ok();
    let addr = ([127, 0, 0, 1], 8080).into();
    let builder = Server::bind(&addr);
    let server = builder.serve(move || {
        service_fn(move |req| microservice_handler(req, &files))
    });
    let server = server.map_err(drop);
    hyper::rt::run(server);
}
