use actix::{Addr};
use actix_web::{
    client, middleware, server, fs, App, Error, Form, HttpMessage,
    HttpRequest, HttpResponse, FutureResponse, Result,
};
use actix_web::http::{self, header, StatusCode};
use actix_web::middleware::{Finished, Middleware, Response, Started};
use actix_web::middleware::identity::RequestIdentity;
use actix_web::middleware::identity::{CookieIdentityPolicy, IdentityService};
use failure::format_err;
use futures::{IntoFuture, Future};
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
    println!("URL: {}", url);
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
    let fut = post_request(&req.state().links.signup, params.into_inner())
        .map(|_: ()| {
            HttpResponse::Found()
            .header(header::LOCATION, "/login.html")
            .finish()
        });
    Box::new(fut)
}

fn signin((req, params): (HttpRequest<State>, Form<UserForm>)) -> FutureResponse<HttpResponse> {
    let fut = post_request(&req.state().links.signin, params.into_inner())
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
            post_request::<_, ()>(&req.state().links.new_comment, params)
                .map(move |_| req)
        })
        .and_then(|req| {
            let res = HttpResponse::build_from(&req)
                .status(StatusCode::FOUND)
                .header(header::LOCATION, "/comments.html")
                .finish();
            Ok(res)
        });
    Box::new(fut)
}

fn comments(req: HttpRequest<State>) -> FutureResponse<HttpResponse> {
    let fut = get_request(&req.state().links.comments)
        .map(|data| {
            HttpResponse::Ok().body(data)
        });
    Box::new(fut)
}

fn counter(req: HttpRequest<State>) -> String {
    format!("{}", req.state().counter.borrow())
}

#[derive(Clone)]
struct LinksMap {
    signup: String,
    signin: String,
    new_comment: String,
    comments: String,
}

#[derive(Clone)]
struct State {
    counter: RefCell<i64>,
    links: LinksMap,
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

fn start(links: LinksMap) {
    let sys = actix::System::new("router");

    let state = State {
        counter: RefCell::default(),
        links,
    };

    let server = server::new(move || {
        App::with_state(state.clone())
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

    sys.run();
}

#[cfg(test)]
mod tests {
    use crate::{start, LinksMap, UserForm};
    use lazy_static::lazy_static;
    use mockito::mock;
    use reqwest::Client;
    use serde::{Deserialize, Serialize};
    use std::mem;
    use std::sync::Mutex;
    use std::time::Duration;
    use std::thread;

    lazy_static! {
        static ref STARTED: Mutex<bool> = Mutex::new(false);
    }

    fn mock_url(base: &str, path: &str) -> String {
        format!("{}{}", base, path)
    }

    fn test_url(path: &str) -> String {
        format!("http://127.0.0.1:8080/api{}", path)
    }

    fn setup() {
        let mut started = STARTED.lock().unwrap();
        if !*started {
            thread::spawn(|| {
                // Mocks have to be initialized from the same thread
                let url = mockito::server_url();
                let _signup = mock("POST", "/signup")
                    .with_status(200)
                    .with_header("Content-Type", "application/json")
                    .with_body("null")
                    .create();
                let _signin = mock("POST", "/signin")
                    .with_status(200)
                    .with_header("Content-Type", "application/json")
                    .with_body(r#"{"id": "user-id"}"#)
                    .create();
                let _new_comment = mock("POST", "/new_comment")
                    .with_status(200)
                    .with_header("Content-Type", "application/json")
                    .with_body("null")
                    .create();
                let _comment = mock("GET", "/comments")
                    .with_status(200)
                    .with_header("Content-Type", "application/json")
                    .with_body(r#"[]"#)
                    .create();
                let links = LinksMap {
                    signup: mock_url(&url, "/signup"),
                    signin: mock_url(&url, "/signin"),
                    new_comment: mock_url(&url, "/new_comment"),
                    comments: mock_url(&url, "/comments"),
                };
                start(links);
            });
            thread::sleep(Duration::from_secs(5));
            *started = true;
        }
    }

    fn test_post<T>(path: &str, data: &T)
    where
        //T: for <'de> Deserialize<'de>,
        T: Serialize,
    {
        setup();
        let client =  Client::new();
        let mut resp = client.post(&test_url(path))
            .form(data)
            .send()
            .unwrap();
        let status = resp.status();
        println!("Response {:?}: {}", status, resp.text().unwrap());
        assert!(status.is_success());
    }

    #[test]
    fn test_signup_with_client() {
        let user = UserForm {
            email: "abc@example.com".into(),
            password: "abc".into(),
        };
        test_post("/signup", &user);
    }

    #[test]
    fn test_signin_with_client() {
        let user = UserForm {
            email: "abc@example.com".into(),
            password: "abc".into(),
        };
        test_post("/signin", &user);
    }

    #[test]
    fn test_list_with_client() {
        let _m = mock("GET", "/list")
            .with_status(200)
            .with_header("Content-Type", "application/json")
            .create();
    }
}
