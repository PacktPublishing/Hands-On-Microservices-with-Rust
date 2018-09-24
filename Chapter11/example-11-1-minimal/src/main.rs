extern crate actix_web;
use actix_web::{http, server, App, HttpRequest, Responder};

fn index(_: &HttpRequest) -> impl Responder {
    format!("Rust Microservice (actix)")
}

fn main() {
    server::new(
        || App::new()
            .resource("/", |r| r.method(http::Method::GET).f(index)))
        .bind("127.0.0.1:8080").unwrap()
        .run();
}
