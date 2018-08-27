extern crate rand;
extern crate futures;
extern crate tokio;
extern crate hyper;

use std::fs;
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use futures::{future, Future, Stream};
use tokio::fs::File;
use hyper::{Body, Error, Method, Request, Response, Server, StatusCode};
use hyper::service::service_fn;

static FILES: &str = "./files";
static INDEX: &[u8] = b"Images Microservice";

fn microservice_handler(req: Request<Body>)
    -> Box<Future<Item=Response<Body>, Error=Error> + Send>
{
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => {
            Box::new(future::ok(Response::new(INDEX.into())))
        },
        (&Method::POST, "/upload") => {
            let name: String = thread_rng().sample_iter(&Alphanumeric).take(20).collect();
            //let file = File::create();
            let body = req.into_body().concat2()
                .map(|chunks| {
                    println!("{}", String::from_utf8_lossy(chunks.as_ref()));
                    Response::new(name.into())
                });
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
    fs::create_dir(FILES).ok();
    let addr = ([127, 0, 0, 1], 8080).into();
    let builder = Server::bind(&addr);
    let server = builder.serve(|| {
        service_fn(microservice_handler)
    });
    let server = server.map_err(drop);
    hyper::rt::run(server);
}
