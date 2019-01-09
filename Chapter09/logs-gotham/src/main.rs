extern crate gotham;
#[macro_use]
extern crate gotham_derive;
extern crate hyper;
extern crate mime;

use failure::{Error, format_err};
use futures::{Future, Stream};
use gotham::state::{FromState, State};
use tokio::net::TcpStream;
use tokio::runtime::Runtime;
use tokio_postgres::{Client, Connection, NoTls};

const HELLO_WORLD: &'static str = "Hello World!";

#[derive(StateData, Eq, PartialEq, Debug)]
struct ConnState {
}

pub fn say_hello(state: State) -> (State, &'static str) {
    //let conn = ConnState::borrow_from(&state);
    (state, HELLO_WORLD)
}

fn connect(
    s: &str,
) -> impl Future<Item = (Client, Connection<TcpStream>), Error = tokio_postgres::Error> {
    let builder = s.parse::<tokio_postgres::Config>().unwrap();
    TcpStream::connect(&"127.0.0.1:5432".parse().unwrap())
        .map_err(|e| panic!("{}", e))
        .and_then(move |s| builder.connect_raw(s, NoTls))
}

pub fn main() -> Result<(), Error> {
    let mut runtime = Runtime::new()?;

    let handshake = connect("user=postgres dbname=postgres");
    let (mut client, connection) = runtime.block_on(handshake)?;
    runtime.spawn(connection.map_err(drop));

    let prepare = client.prepare("SELECT 1::INT4");
    let statement = runtime.block_on(prepare).unwrap();
    println!("HERE!");
    let select = client.query(&statement, &[]).collect().map(|rows| {
        println!("{:?}", rows[0].get::<_, i32>(0));
    });
    let res = runtime.block_on(select).unwrap();
    println!("{:?}", res);

    let addr = "127.0.0.1:7878";
    println!("Listening for requests at http://{}", addr);
    gotham::start_on_executor(addr, || Ok(say_hello), runtime.executor());
    runtime
        .shutdown_on_idle()
        .wait()
        .map_err(|()| format_err!("can't wait for the runtime"))
}
