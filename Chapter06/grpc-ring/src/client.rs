use crate::ring::Empty;
use crate::ring_grpc::{Ring, RingClient, RingServer};
use grpc::{ClientConf, ClientStubExt, Error as GrpcError, ServerBuilder, SingleResponse, RequestOptions};
use std::net::SocketAddr;

pub struct Remote {
    client: RingClient,
}

impl Remote {
    pub fn new(addr: SocketAddr) -> Result<Self, GrpcError> {
        let host = addr.ip().to_string();
        let port = addr.port();
        let conf = ClientConf::default();
        let client = RingClient::new_plain(&host, port, conf)?;
        Ok(Self {
            client
        })
    }

    pub fn start_roll_call(&self) -> Result<Empty, GrpcError> {
        self.client.start_roll_call(RequestOptions::new(), Empty::new())
            .wait()
            .map(|(_, value, _)| value)
    }

    pub fn mark_itself(&self) -> Result<Empty, GrpcError> {
        self.client.mark_itself(RequestOptions::new(), Empty::new())
            .wait()
            .map(|(_, value, _)| value)
    }
}
