extern crate actix_web;

use std::cell::RefCell;
use actix_web::{fs, server, App, HttpRequest, HttpResponse, Responder};
use actix_web::http::{self, header, Method};

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

fn value(req: HttpRequest<AppState>) -> impl Responder {
    format!("Counter: {}", req.state().0.borrow())
}

fn main() {
    server::new(|| {
        App::with_state(AppState::new())
            .scope("/count", |scope| {
                scope.nested("/v1", |scope| {
                    scope
                        .route("/up", http::Method::GET, count_up)
                        .route("/down", http::Method::GET, count_down)
                })
                .route("/value", http::Method::GET, value)
            })
            .handler("/static", fs::StaticFiles::new("static").unwrap())
            .resource("/", |r| r.method(Method::GET).f(|_| {
                HttpResponse::Found()
                    .header(header::LOCATION, "static/index.html")
                    .finish()
}))
    })
    .bind("127.0.0.1:8080").unwrap().run();
}
