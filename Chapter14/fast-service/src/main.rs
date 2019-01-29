use actix_web::{middleware, server, App, HttpRequest, HttpResponse};
use askama::Template;
use chrono::Utc;
use std::sync::{Arc, Mutex, MutexGuard, RwLock, RwLockReadGuard};
use std::time::Duration;
use std::thread;

#[cfg(feature="borrow")]
#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    time: &'a str,
}

#[cfg(not(feature="borrow"))]
#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    time: String,
}

#[derive(Clone)]
struct State {
    #[cfg(not(feature="rwlock"))]
    last_minute: Arc<Mutex<String>>,
    #[cfg(feature="rwlock")]
    last_minute: Arc<RwLock<String>>,
    cached: Arc<RwLock<Option<String>>>,
}

fn index(req: &HttpRequest<State>) -> HttpResponse {
    if cfg!(feature="cache") {
        let cached = req.state().cached.read().unwrap();
        if let Some(ref body) = *cached {
            return HttpResponse::Ok().body(body.to_owned());
        }
    }

    #[cfg(not(feature="rwlock"))]
    let last_minute = req.state().last_minute.lock().unwrap();
    #[cfg(feature="rwlock")]
    let last_minute = req.state().last_minute.read().unwrap();

    #[cfg(not(feature="borrow"))]
    let template = IndexTemplate { time: last_minute.to_owned() };
    #[cfg(feature="borrow")]
    let template = IndexTemplate { time: &last_minute };

    let body = template.render().unwrap();
    if cfg!(feature="cache") {
        let mut cached = req.state().cached.write().unwrap();
        *cached = Some(body.clone());
    }
    HttpResponse::Ok().body(body)
}

fn now() -> String {
    Utc::now().to_string()
}

fn main() {
    env_logger::init();
    let sys = actix::System::new("fast-service");

    let value = now();
    #[cfg(not(feature="rwlock"))]
    let last_minute = Arc::new(Mutex::new(value));
    #[cfg(feature="rwlock")]
    let last_minute = Arc::new(RwLock::new(value));

    let last_minute_ref = last_minute.clone();
    thread::spawn(move || {
        loop {
            {
                #[cfg(not(feature="rwlock"))]
                let mut last_minute = last_minute_ref.lock().unwrap();
                #[cfg(feature="rwlock")]
                let mut last_minute = last_minute_ref.write().unwrap();
                *last_minute = now();
            }
            thread::sleep(Duration::from_secs(3));
        }
    });

    let cached = Arc::new(RwLock::new(None));
    let state = State {
        last_minute,
        cached,
    };
    server::new(move || {
        App::with_state(state.clone())
            .middleware(middleware::Logger::default())
            .resource("/", |r| r.f(index))
    })
    .bind("127.0.0.1:8080")
    .unwrap()
    .start();

    let _ = sys.run();
}
