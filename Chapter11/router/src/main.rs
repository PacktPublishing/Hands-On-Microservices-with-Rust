extern crate actix;
extern crate actix_web;
extern crate env_logger;
extern crate futures;

use actix_web::{
    client, middleware, server, fs, App, AsyncResponder, Body, Error, HttpMessage,
    HttpRequest, HttpResponse,
};
use actix_web::http::header;
use actix_web::middleware::session;
use futures::{Future, Stream};

fn request(url: &str) -> impl Future<Item = HttpResponse, Error = Error> {
    client::ClientRequest::post(url)
        .finish().unwrap()
        .send()
        .map_err(Error::from)
        .and_then(
            |resp| resp.body()
                .from_err()
                .and_then(|body| {
                    Ok(HttpResponse::Ok().body(body))
                }))
        .responder()
}

fn signup(_req: &HttpRequest) -> Box<Future<Item = HttpResponse, Error = Error>> {
    let fut = request("http://127.0.0.1:8000/signup")
        .map(|_| {
            HttpResponse::Found()
            .header(header::LOCATION, "/login.html")
            .finish()
        });
    Box::new(fut)
}

fn signin(_req: &HttpRequest) -> Box<Future<Item = HttpResponse, Error = Error>> {
    Box::new(request("http://127.0.0.1:8000/signin"))
}

/*
/// streaming client request to a streaming server response
fn streaming(_req: &HttpRequest) -> Box<Future<Item = HttpResponse, Error = Error>> {
    client::ClientRequest::get("https://www.rust-lang.org/en-US/")
        .finish().unwrap()
        .send()
        .map_err(Error::from)
        .and_then(|resp| {
            Ok(HttpResponse::Ok()
               .body(Body::Streaming(Box::new(resp.payload().from_err()))))
        })
        .responder()
}
*/

fn main() {
    ::std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
    let sys = actix::System::new("router");

    server::new(|| {
        App::new()
            .middleware(middleware::Logger::default())
            .middleware(session::SessionStorage::new(
                session::CookieSessionBackend::signed(&[0; 32]).secure(false)
            ))
            //.resource("/streaming", |r| r.f(streaming))
            .resource("/signup", |r| r.f(signup))
            .resource("/signin", |r| r.f(signin))
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
