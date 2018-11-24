extern crate actix;
extern crate actix_web;
extern crate config;
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
use log::{debug, error};
use serde::{Deserialize, Serialize};
use serde_derive::{Deserialize, Serialize};
use std::sync::Arc;

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

fn signup((req, params): (HttpRequest<State>, Form<UserForm>)) -> FutureResponse<HttpResponse> {
    let url = format!("{}/signup", req.state().users());
    let fut = request(&url, params.into_inner())
        .map(|_: ()| {
            HttpResponse::Found()
            .header(header::LOCATION, "/login.html")
            .finish()
        });
    Box::new(fut)
}

fn signin((req, params): (HttpRequest<State>, Form<UserForm>)) -> FutureResponse<HttpResponse> {
    let url = format!("{}/signin", req.state().users());
    let fut = request(&url, params.into_inner())
        .map(move |id: UserId| {
            req.remember(id.id);
            HttpResponse::build_from(&req)
            .status(StatusCode::FOUND)
            .header(header::LOCATION, "/comments.html")
            .finish()
        });
    Box::new(fut)
}

fn new_comment((req, params): (HttpRequest<State>, Form<AddComment>)) -> FutureResponse<HttpResponse> {
    let url = format!("{}/new_comment", req.state().content());
    let fut = req.identity()
        .ok_or(format_err!("not authorized").into())
        .into_future()
        .and_then(move |uid| {
            let params = NewComment {
                uid,
                text: params.into_inner().text,
            };
            request::<_, ()>(&url, params)
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

fn comments(req: HttpRequest<State>) -> FutureResponse<HttpResponse> {
    let url = format!("{}/list", req.state().content());
    let fut = get_req(&url)
        .map(|data| {
            HttpResponse::Ok().body(data)
        });
    Box::new(fut)
}

#[derive(Deserialize)]
struct Config {
    address: Option<String>,
    users: Option<String>,
    content: Option<String>,
}

#[derive(Clone)]
struct State {
    users: Arc<String>,
    content: Arc<String>,
}

impl State {
    fn users(&self) -> &str {
        self.users.as_ref()
    }

    fn content(&self) -> &str {
        self.content.as_ref()
    }
}

fn main() -> Result<(), failure::Error> {
    env_logger::init();
    let mut config = config::Config::default();
    config
        .merge(config::File::with_name("config"))?
        .merge(config::Environment::with_prefix("ROUTER"))?;
    let config: Config = config.try_into()?;
    let sys = actix::System::new("router");

    let users = config.users.unwrap_or("http://127.0.0.1:8001".into());
    let content = config.content.unwrap_or("http://127.0.0.1:8003".into());
    let state = State {
        users: Arc::new(users),
        content: Arc::new(content),
    };
    let address = config.address.unwrap_or("127.0.0.1:8080".into());
    server::new(move || {
        App::with_state(state.clone())
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
        .bind(&address)
        .unwrap()
        .start();

    debug!("Started http server: {}", address);
    let _ = sys.run();
    Ok(())
}
