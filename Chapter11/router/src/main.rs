extern crate actix;
extern crate actix_web;
extern crate env_logger;
extern crate failure;
extern crate futures;
extern crate serde;
extern crate serde_derive;
extern crate serde_urlencoded;

use actix_web::{
    client, middleware, server, fs, App, AsyncResponder, Body, Error, Form, HttpMessage,
    HttpRequest, HttpResponse,
};
use actix_web::http::{self, header};
use actix_web::middleware::session;
use actix_web::middleware::identity::RequestIdentity;
use actix_web::middleware::identity::{CookieIdentityPolicy, IdentityService};
use failure::format_err;
use futures::{IntoFuture, Future, Stream};
use serde::{Deserialize, Serialize};
use serde_derive::{Deserialize, Serialize};

fn boxed<I, E, F>(fut: F) -> Box<Future<Item = I, Error = E>>
where
    F: Future<Item = I, Error = E> + 'static,
{
    Box::new(fut)
}

fn request<T>(url: &str, params: T) -> impl Future<Item = Vec<u8>, Error = Error>
where
    T: Serialize,
{
    client::ClientRequest::post(url)
        .form(params).into_future()
        .and_then(|req| {
            req.send()
                .map_err(Error::from)
                .and_then(|resp| {
                    if resp.status().is_success() {
                        let fut = resp
                            .body()
                            .from_err();
                        boxed(fut)
                    } else {
                        let fut = Err(format_err!("microservice error"))
                            .into_future()
                            .from_err();
                        boxed(fut)
                    }
                })
                .map(|bytes| bytes.to_vec())
        })
}


#[derive(Deserialize, Serialize)]
pub struct UserForm {
    email: String,
    password: String,
}

fn signup(params: Form<UserForm>) -> Box<Future<Item = HttpResponse, Error = Error>> {
    let fut = request("http://127.0.0.1:8000/signup", params.into_inner())
        .map(|_| {
            HttpResponse::Found()
            .header(header::LOCATION, "/login.html")
            .finish()
        });
    Box::new(fut)
}

fn signin(params: Form<UserForm>) -> Box<Future<Item = HttpResponse, Error = Error>> {
    let fut = request("http://127.0.0.1:8000/signin", params.into_inner())
        .map(|_| {
            HttpResponse::Found()
            .header(header::LOCATION, "/comments.html")
            .finish()
        });
    Box::new(fut)
}

fn main() {
    ::std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
    let sys = actix::System::new("router");

    server::new(|| {
        App::new()
            .middleware(middleware::Logger::default())
            .middleware(IdentityService::new(
                    CookieIdentityPolicy::new(&[0; 32])
                    .name("auth-example")
                    .secure(false),
                    ))
            .resource("/signup", |r| {
                r.method(http::Method::POST).with(signup)
            })
            .resource("/signin", |r| {
                r.method(http::Method::POST).with(signin)
            })
            .handler(
                "/",
                fs::StaticFiles::new("./static/").unwrap().index_file("index.html")
            )
    }).workers(1)
        .bind("127.0.0.1:8080")
        .unwrap()
        .start();

    println!("Started http server: 127.0.0.1:8080");
    let _ = sys.run();
}
