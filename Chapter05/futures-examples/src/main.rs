extern crate failure;
extern crate futures;
extern crate tokio;

use std::io;
use failure::Error;
use futures::{future, stream, Future, Stream, Sink, IntoFuture};
use futures::sync::{mpsc, oneshot};
use tokio::net::{UdpSocket, UdpFramed};
use tokio::codec::LinesCodec;

fn to_box<T>(fut :T) -> Box<dyn Future<Item=(), Error=()> + Send>
where
    T: IntoFuture,
    T::Future: Send + 'static,
    T::Item: 'static,
    T::Error: 'static,
{
    let fut = fut.into_future().map(drop).map_err(drop);
    Box::new(fut)
}

fn other<E>(err: E) -> io::Error
where
    E: Into<Box<std::error::Error + Send + Sync>>,
{
    io::Error::new(io::ErrorKind::Other, err)
}

fn main() {
    single();
    multiple();
    send_spawn();
    println!("Start UDP echo");
    alt_udp_echo().unwrap();
}

fn single() {
    let (tx_sender, rx_future) = oneshot::channel::<u8>();
    let receiver = rx_future.map(|x| {
        println!("Received: {}", x);
    });
    let sender = tx_sender.send(8);
    let execute_all = future::join_all(vec![
        to_box(receiver),
        to_box(sender),
    ]).map(drop);
    tokio::run(execute_all);
}

fn multiple() {
    let (tx_sink, rx_stream) = mpsc::channel::<u8>(8);
    let receiver = rx_stream.fold(0, |acc, value| {
        future::ok(acc + value)
    }).map(|x| {
        println!("Calculated: {}", x);
    });
    let send_1 = tx_sink.clone().send(1);
    let send_2 = tx_sink.clone().send(2);
    let send_3 = tx_sink.clone().send(3);
    let execute_all = future::join_all(vec![
        to_box(receiver),
        to_box(send_1),
        to_box(send_2),
        to_box(send_3),
    ]).map(drop);
    drop(tx_sink);
    tokio::run(execute_all);
}

fn alt_udp_echo() -> Result<(), Error> {
    let from = "0.0.0.0:12345".parse()?;
    let socket = UdpSocket::bind(&from)?;
    let framed = UdpFramed::new(socket, LinesCodec::new());
    let (sink, stream) = framed.split();
    let (tx, rx) = mpsc::channel(16);
    let rx = rx.map_err(|_| other("can't take a message"))
        .fold(sink, |sink, frame| {
            sink.send(frame)
        });
    let process = stream.and_then(move |args| {
        tx.clone()
            .send(args)
            .map(drop)
            .map_err(other)
    }).collect();
    let execute_all = future::join_all(vec![
        to_box(rx),
        to_box(process),
    ]).map(drop);
    Ok(tokio::run(execute_all))
}

fn send_spawn() {
    let (tx_sink, rx_stream) = mpsc::channel::<u8>(8);
    let receiver = rx_stream.fold(0, |acc, value| {
        println!("Received: {}", value);
        future::ok(acc + value)
    }).map(drop);
    let spawner = stream::iter_ok::<_, ()>(1u8..11u8).map(move |x| {
        let fut = tx_sink.clone().send(x).map(drop).map_err(drop);
        tokio::spawn(fut);
    }).collect();
    let execute_all = future::join_all(vec![
        to_box(spawner),
        to_box(receiver),
    ]).map(drop);
    tokio::run(execute_all);
}
