extern crate hyper;
extern crate ntp;

use hyper::{Body, Response, Server};
use hyper::rt::Future;
use hyper::service::service_fn_ok;

// TODO Convert to UDP-provider microservice!

fn main() {
    let addr = ([127, 0, 0, 1], 8080).into();
    let builder = Server::bind(&addr);
    let server = builder.serve(|| {
        service_fn_ok(|_| {
            let address = "0.pool.ntp.org:123";
            // TODO Use thread pool
            let response: ntp::packet::Packet = ntp::request(address).unwrap();
            let unix_time = ntp::unix_time::Instant::from(timestamp);
            chrono::Local.timestamp(unix_time.secs(), unix_time.subsec_nanos() as _)
            Response::new(Body::from(format!("{:?}", response)))
        })
    });
    let server = server.map_err(drop);
    hyper::rt::run(server);
}
