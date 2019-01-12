mod cache;

use actix::{Addr, SyncArbiter};
use actix_web::{
    client, middleware, server, fs, App, Error, Form, HttpMessage,
    HttpRequest, HttpResponse, FutureResponse, Result,
};
use actix_web::http::{self, header, StatusCode};
use actix_web::middleware::{Finished, Middleware, Response, Started};
use actix_web::middleware::identity::RequestIdentity;
use actix_web::middleware::identity::{CookieIdentityPolicy, IdentityService};
use crate::cache::{CacheActor, GetValue, SetValue};
use failure::format_err;
use futures::{IntoFuture, Future, future};
use log::{error};
use serde::{Deserialize, Serialize};
use serde_derive::{Deserialize, Serialize};
use std::cell::RefCell;

fn boxed<I, E, F>(fut: F) -> Box<Future<Item = I, Error = E>>
where
    F: Future<Item = I, Error = E> + 'static,
{
    Box::new(fut)
}

fn get_request(url: &str) -> impl Future<Item = Vec<u8>, Error = Error> {
    client::ClientRequest::get(url)
        .finish().into_future()
        .and_then(|req| {
            req.send()
                .map_err(Error::from)
                .and_then(|resp| resp.body().from_err())
                .map(|bytes| bytes.to_vec())
        })
}

fn post_request<T, O>(url: &str, params: T) -> impl Future<Item = O, Error = Error>
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
    let fut = post_request("http://127.0.0.1:8001/signup", params.into_inner())
        .map(|_: ()| {
            HttpResponse::Found()
            .header(header::LOCATION, "/login.html")
            .finish()
        });
    Box::new(fut)
}

fn signin((req, params): (HttpRequest<State>, Form<UserForm>)) -> FutureResponse<HttpResponse> {
    let fut = post_request("http://127.0.0.1:8001/signin", params.into_inner())
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
    let fut = req.identity()
        .ok_or(format_err!("not authorized").into())
        .into_future()
        .and_then(move |uid| {
            let params = NewComment {
                uid,
                text: params.into_inner().text,
            };
            post_request::<_, ()>("http://127.0.0.1:8003/new_comment", params)
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
    let fut = get_request("http://127.0.0.1:8003/list");
    let fut = cache(req.state(), "/list", fut)
        .map(|data| {
            HttpResponse::Ok().body(data)
        });
    Box::new(fut)
}

fn counter(req: HttpRequest<State>) -> String {
    format!("{}", req.state().counter.borrow())
}

fn cache<F>(state: &State, path: &str, fut: F)
    -> impl Future<Item = Vec<u8>, Error = Error>
where
    F: Future<Item = Vec<u8>, Error = Error>,
{
    let path = path.to_owned();
    let addr = state.cache.clone();
    get_value(addr.clone(), &path)
        .from_err::<Error>()
        .or_else(move |_| fut)
        .and_then(move |data| {
            set_value(addr, &path, &data)
                .then(move |_| future::ok::<_, Error>(data))
                .from_err::<Error>()
        })
}


struct State {
    counter: RefCell<i64>,
    cache: Addr<CacheActor>,
}

fn get_value(addr: Addr<CacheActor>, path: &str) -> Box<Future<Item = Vec<u8>, Error = failure::Error>> {
    let msg = GetValue {
        path: path.to_owned(),
    };
    let fut = addr.send(msg)
        .from_err::<failure::Error>()
        .and_then(|x| x.map_err(failure::Error::from));
    Box::new(fut)
}

fn set_value(addr: Addr<CacheActor>, path: &str, value: &[u8]) -> Box<Future<Item = (), Error = failure::Error>> {
    let msg = SetValue {
        path: path.to_owned(),
        content: value.to_owned(),
    };
    let fut = addr.send(msg)
        .from_err::<failure::Error>()
        .and_then(|x| x.map_err(failure::Error::from));
    Box::new(fut)
}

impl State {
    fn new(cache: Addr<CacheActor>) -> Self {
        Self {
            counter: RefCell::default(),
            cache,
        }
    }
}

pub struct Counter;

impl Middleware<State> for Counter {
    fn start(&self, req: &HttpRequest<State>) -> Result<Started> {
        let value = *req.state().counter.borrow();
        *req.state().counter.borrow_mut() = value + 1;
        Ok(Started::Done)
    }

    fn response(&self, _req: &HttpRequest<State>, resp: HttpResponse) -> Result<Response> {
        Ok(Response::Done(resp))
    }

    fn finish(&self, _req: &HttpRequest<State>, _resp: &HttpResponse) -> Finished {
        Finished::Done
    }
}

fn main() {
    env_logger::init();
    let sys = actix::System::new("router");

    let addr = SyncArbiter::start(3, || {
        CacheActor::new("redis://127.0.0.1/", 10)
    });

    server::new(move || {
        let state = State::new(addr.clone());
        App::with_state(state)
            .middleware(middleware::Logger::default())
            .middleware(IdentityService::new(
                    CookieIdentityPolicy::new(&[0; 32])
                    .name("auth-example")
                    .secure(false),
                    ))
            .middleware(Counter)
            .scope("/api", |scope| {
                scope
                    .route("/signup", http::Method::POST, signup)
                    .route("/signin", http::Method::POST, signin)
                    .route("/new_comment", http::Method::POST, new_comment)
                    .route("/comments", http::Method::GET, comments)
            })
            .route("/stats/counter", http::Method::GET, counter)
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
