extern crate actix;
extern crate actix_web;
extern crate env_logger;
extern crate failure;
extern crate futures;
extern crate log;
extern crate serde;
extern crate serde_derive;
extern crate serde_urlencoded;

use actix_web::{
    client, middleware, server, fs, App, Error, Form, HttpMessage,
    HttpRequest, HttpResponse, FutureResponse,
};
use actix_web::http::{self, header, StatusCode};
use actix_web::middleware::identity::RequestIdentity;
use actix_web::middleware::identity::{CookieIdentityPolicy, IdentityService};
use failure::format_err;
use futures::{IntoFuture, Future};
use log::{error};
use serde::{Deserialize, Serialize};
use serde_derive::{Deserialize, Serialize};

fn boxed<I, E, F>(fut: F) -> Box<Future<Item = I, Error = E>>
where
    F: Future<Item = I, Error = E> + 'static,
{
    Box::new(fut)
}

fn get_req(url: &str) -> impl Future<Item = Vec<u8>, Error = Error> {
    client::ClientRequest::get(url)
        .finish().into_future()
        .and_then(|req| {
            req.send()
                .map_err(Error::from)
                .and_then(|resp| resp.body().from_err())
                .map(|bytes| bytes.to_vec())
        })
}

fn request<T, O>(url: &str, params: T) -> impl Future<Item = O, Error = Error>
where
    T: Serialize,
    O: for <'de> Deserialize<'de> + 'static,
{
    client::ClientRequest::post(url)
        .form(params)
        .into_future()
        .and_then(|req| {
            req.send()
                .map_err(Error::from)
                .and_then(|resp| {
                    if resp.status().is_success() {
                        let fut = resp
                            .json::<O>()
                            .from_err();
                        boxed(fut)
                    } else {
                        error!("Microservice error: {}", resp.status());
                        let fut = Err(format_err!("microservice error"))
                            .into_future()
                            .from_err();
                        boxed(fut)
                    }
                })
        })
}


#[derive(Deserialize, Serialize)]
pub struct UserForm {
    email: String,
    password: String,
}

#[derive(Deserialize)]
pub struct UserId {
    id: String,
}

#[derive(Deserialize, Serialize)]
pub struct Comment {
    pub id: Option<i32>,
    pub uid: String,
    pub text: String,
}

#[derive(Deserialize)]
pub struct AddComment {
    pub text: String,
}

#[derive(Serialize)]
pub struct NewComment {
    pub uid: String,
    pub text: String,
}

fn signup(params: Form<UserForm>) -> FutureResponse<HttpResponse> {
    let fut = request("http://127.0.0.1:8001/signup", params.into_inner())
        .map(|_: ()| {
            HttpResponse::Found()
            .header(header::LOCATION, "/login.html")
            .finish()
        });
    Box::new(fut)
}

fn signin((req, params): (HttpRequest, Form<UserForm>)) -> FutureResponse<HttpResponse> {
    let fut = request("http://127.0.0.1:8001/signin", params.into_inner())
        .map(move |id: UserId| {
            req.remember(id.id);
            HttpResponse::build_from(&req)
            .status(StatusCode::FOUND)
            .header(header::LOCATION, "/comments.html")
            .finish()
        });
    Box::new(fut)
}

fn new_comment((req, params): (HttpRequest, Form<AddComment>)) -> FutureResponse<HttpResponse> {
    let fut = req.identity()
        .ok_or(format_err!("not authorized").into())
        .into_future()
        .and_then(move |uid| {
            let params = NewComment {
                uid,
                text: params.into_inner().text,
            };
            request::<_, ()>("http://127.0.0.1:8003/new_comment", params)
        })
        .then(move |_| {
            let res = HttpResponse::build_from(&req)
                .status(StatusCode::FOUND)
                .header(header::LOCATION, "/comments.html")
                .finish();
            Ok(res)
        });
    Box::new(fut)
}

fn comments(_req: HttpRequest) -> FutureResponse<HttpResponse> {
    let fut = get_req("http://127.0.0.1:8003/list")
        .map(|data| {
            HttpResponse::Ok().body(data)
        });
    Box::new(fut)
}

fn main() {
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
            .scope("/api", |scope| {
                scope
                    .route("/signup", http::Method::POST, signup)
                    .route("/signin", http::Method::POST, signin)
                    .route("/new_comment", http::Method::POST, new_comment)
                    .route("/comments", http::Method::GET, comments)
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
