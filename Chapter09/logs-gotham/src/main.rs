extern crate gotham;
#[macro_use]
extern crate gotham_derive;
extern crate hyper;
extern crate mime;

use failure::{Error, format_err};
use futures::{Future, Stream};
use futures::future;
use gotham::handler::{IntoHandlerFuture, HandlerFuture};
use gotham::middleware::state::StateMiddleware;
use gotham::pipeline::single::single_pipeline;
use gotham::pipeline::single_middleware;
use gotham::router::Router;
use gotham::router::builder::{DefineSingleRoute, DrawRoutes, build_router};
use gotham::state::{FromState, State};
use std::sync::{Arc, Mutex};
use tokio::net::TcpStream;
use tokio::runtime::Runtime;
use tokio_postgres::{Client, Connection, NoTls};

const HELLO_WORLD: &'static str = "Hello World!";

#[derive(Clone, StateData)]
struct ConnState {
    client: Arc<Mutex<Client>>,
}

impl ConnState {
    fn new(client: Client) -> Self {
        Self {
            client: Arc::new(Mutex::new(client)),
        }
    }
}

/*
pub fn say_hello(state: State) -> (State, &'static str) {
    //let conn = ConnState::borrow_from(&state);
    (state, HELLO_WORLD)
}
*/

fn connect(
    s: &str,
) -> impl Future<Item = (Client, Connection<TcpStream>), Error = tokio_postgres::Error> {
    let builder = s.parse::<tokio_postgres::Config>().unwrap();
    TcpStream::connect(&"127.0.0.1:5432".parse().unwrap())
        .map_err(|e| panic!("{}", e))
        .and_then(move |s| builder.connect_raw(s, NoTls))
}

fn say_hello(mut state: State) -> Box<HandlerFuture> {
    let conn = ConnState::borrow_from(&state);
    let fut = (state, HELLO_WORLD).into_handler_future();
    Box::new(fut)
}

fn router(state: ConnState) -> Router {
    let middleware = StateMiddleware::new(state);
    let pipeline = single_middleware(middleware);
    let (chain, pipelines) = single_pipeline(pipeline);
    build_router(chain, pipelines, |route| {
        route.get("/").to(say_hello);
    })
}

pub fn main() -> Result<(), Error> {
    let mut runtime = Runtime::new()?;

    let handshake = connect("user=postgres dbname=postgres");
    let (mut client, connection) = runtime.block_on(handshake)?;
    runtime.spawn(connection.map_err(drop));

    let prepare = client.prepare("SELECT 1::INT4");
    let statement = runtime.block_on(prepare).unwrap();
    let select = client.query(&statement, &[]).collect().map(|rows| {
        println!("{:?}", rows[0].get::<_, i32>(0));
    });
    let res = runtime.block_on(select).unwrap();
    println!("{:?}", res);

    let state = ConnState::new(client);
    let router = router(state);

    let addr = "127.0.0.1:7878";
    println!("Listening for requests at http://{}", addr);
    gotham::start_on_executor(addr, router, runtime.executor());
    runtime
        .shutdown_on_idle()
        .wait()
        .map_err(|()| format_err!("can't wait for the runtime"))
}
