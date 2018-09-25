extern crate actix_web;

use std::cell::RefCell;
use actix_web::{http, server, App, HttpRequest, Responder};

struct AppState(RefCell<i64>);

impl AppState {
    fn new() -> Self {
        AppState(RefCell::default())
    }
}

fn count_up(req: HttpRequest<AppState>) -> impl Responder {
    *req.state().0.borrow_mut() += 1;
    format!("Count Up")
}

fn count_down(req: HttpRequest<AppState>) -> impl Responder {
    *req.state().0.borrow_mut() -= 1;
    format!("Count Down")
}

fn index(req: HttpRequest<AppState>) -> impl Responder {
    format!("Rust Microservice (actix): {}", req.state().0.borrow())
}

fn main() {
    server::new(|| {
        App::with_state(AppState::new())
            .route("/", http::Method::GET, index)
            .scope("/count", |scope| {
                scope.nested("/v1", |scope| {
                    scope
                        .route("/up", http::Method::GET, count_up)
                        .route("/down", http::Method::GET, count_down)
                })
            })
    })
    .bind("127.0.0.1:8080").unwrap().run();
}
