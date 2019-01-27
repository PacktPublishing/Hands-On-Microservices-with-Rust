use hyper::{Body, Response, Server};
use hyper::rt::Future;
use hyper::service::service_fn_ok;

fn main() {
    let addr = ([127, 0, 0, 1], 8080).into();
    let builder = Server::bind(&addr);
    let server = builder.serve(|| {
        service_fn_ok(|_| {
            Response::new(Body::from("Rust Microservice"))
        })
    });
    let server = server.map_err(drop);
    hyper::rt::run(server);
}
