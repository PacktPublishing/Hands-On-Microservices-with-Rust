use actix::System;
use actix_web::{http, middleware, server, App, Error as WebError, HttpRequest, HttpResponse};
use failure::Error;
use futures::Future;
use lapin::channel::{BasicProperties, BasicPublishOptions, Channel};
use tokio::net::TcpStream;

#[derive(Clone)]
struct AppState {
    channel: Channel<TcpStream>,
}

fn index(req: HttpRequest<AppState>)
    -> impl Future<Item = HttpResponse, Error = WebError>
{
    let opts = BasicPublishOptions::default();
    let prop = BasicProperties::default().with_delivery_mode(2);
    let data = "content".to_string().into_bytes();
    req.state().channel.basic_publish("", rabbit_actix::QUEUE, data, opts, prop)
        .map_err(|err| WebError::from(Error::from(err)))
        .map(|_| {
            HttpResponse::Ok().body(format!("Sent"))
        })
}

fn main() -> Result<(), Error> {
    env_logger::init();
    let mut sys = System::new("rabbit-actix-server");
    let channel = rabbit_actix::spawn_client(&mut sys)?;
    sys.block_on(rabbit_actix::ensure_queue(&channel))?;

    let state = AppState { channel };
    server::new(move || {
        App::with_state(state.clone())
            .middleware(middleware::Logger::default())
            //.resource("/", |r| r.f(index))
            .resource("/", |r| r.method(http::Method::GET).with_async(index))
    }).bind("127.0.0.1:8080")
    .unwrap()
        .start();

    sys.run();
    Ok(())
}
