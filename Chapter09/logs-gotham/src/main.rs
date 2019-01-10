use failure::{Error, format_err};
use futures::{Future, Stream};
use futures::future;
use gotham::handler::HandlerFuture;
use gotham::middleware::state::StateMiddleware;
use gotham::pipeline::single::single_pipeline;
use gotham::pipeline::single_middleware;
use gotham::router::Router;
use gotham::router::builder::{DefineSingleRoute, DrawRoutes, build_router};
use gotham::state::{FromState, State};
use gotham_derive::StateData;
use hyper::Response;
use hyper::header::{HeaderMap, USER_AGENT};
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;
use tokio_postgres::{Client, NoTls};

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

fn register_user_agent(state: State) -> Box<HandlerFuture> {
    let user_agent = HeaderMap::borrow_from(&state)
        .get(USER_AGENT)
        .map(|value| value.to_str().unwrap().to_string())
        .unwrap_or_else(|| "<undefined>".into());

    let conn = ConnState::borrow_from(&state);
    let client_1 = conn.client.clone();
    let client_2 = conn.client.clone();

    let res = future::ok(())
        .and_then(move |_| {
            let mut client = client_1.lock().unwrap();
            client.prepare("INSERT INTO agents (agent) VALUES ($1)
                            RETURNING agent")
        })
        .and_then(move |statement| {
            let mut client = client_2.lock().unwrap();
            client.query(&statement, &[&user_agent]).collect().map(|rows| {
                rows[0].get::<_, String>(0)
            })
        })
        .then(|res| {
            match res {
                Ok(value) => {
                    let value = format!("SQL: {}", value);
                    Ok((state, Response::new(value.into())))
                }
                Err(err) => {
                    Ok((state, Response::new(err.to_string().into())))
                }
            }
        });

    Box::new(res)
}

fn router(state: ConnState) -> Router {
    let middleware = StateMiddleware::new(state);
    let pipeline = single_middleware(middleware);
    let (chain, pipelines) = single_pipeline(pipeline);
    build_router(chain, pipelines, |route| {
        route.get("/").to(register_user_agent);
    })
}

pub fn main() -> Result<(), Error> {
    let mut runtime = Runtime::new()?;

    let handshake = tokio_postgres::connect("postgres://postgres@localhost:5432", NoTls);
    let (mut client, connection) = runtime.block_on(handshake)?;
    runtime.spawn(connection.map_err(drop));

    let execute = client.batch_execute(
        "CREATE TABLE IF NOT EXISTS agents (
            agent TEXT NOT NULL,
            timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );");
    runtime.block_on(execute)?;

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
