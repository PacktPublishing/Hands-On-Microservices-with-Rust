extern crate failure;
extern crate hyper;

use failure::Error;
use hyper::{Body, Response, Server};
use hyper::rt::Future;
use hyper::service::service_fn_ok;

fn main() -> Result<(), Error> {
    let addr = ([127, 0, 0, 1], 8080).into();
    let service = || {
        service_fn_ok(|_| {
            Response::new(Body::from(""))
        })
    };
    let server = Server::bind(&addr)
        .serve(service)
        .map_err(drop);
    hyper::rt::run(server);
    Ok(())
}
