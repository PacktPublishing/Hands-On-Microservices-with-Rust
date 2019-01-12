use actix::prelude::*;
use failure::Error;
use futures::Future;
use redis::{Commands, Client, RedisError};

pub struct CacheActor {
    client: Client,
    expiration: usize,
}

impl CacheActor {
    pub fn new(addr: &str, expiration: usize) -> Self {
        let client = Client::open(addr).unwrap();
        Self { client, expiration }
    }
}

impl Actor for CacheActor {
    type Context = SyncContext<Self>;
}

struct SetValue {
    pub path: String,
    pub content: Vec<u8>,
}

impl Message for SetValue {
    type Result = Result<(), RedisError>;
}

impl Handler<SetValue> for CacheActor {
    type Result = Result<(), RedisError>;

    fn handle(&mut self, msg: SetValue, _: &mut Self::Context) -> Self::Result {
        self.client.set_ex(msg.path, msg.content, self.expiration)
    }
}

struct GetValue {
    pub path: String,
}

impl Message for GetValue {
    type Result = Result<Option<Vec<u8>>, RedisError>;
}

impl Handler<GetValue> for CacheActor {
    type Result = Result<Option<Vec<u8>>, RedisError>;

    fn handle(&mut self, msg: GetValue, _: &mut Self::Context) -> Self::Result {
        self.client.get(&msg.path)
    }
}

#[derive(Clone)]
pub struct CacheLink {
    addr: Addr<CacheActor>,
}

impl CacheLink {
    pub fn new(addr: Addr<CacheActor>) -> Self {
        Self { addr }
    }

    pub fn get_value(&self, path: &str) -> Box<Future<Item = Option<Vec<u8>>, Error = Error>> {
        let msg = GetValue {
            path: path.to_owned(),
        };
        let fut = self.addr.send(msg)
            .from_err::<Error>()
            .and_then(|x| x.map_err(Error::from));
        Box::new(fut)
    }

    pub fn set_value(&self, path: &str, value: &[u8]) -> Box<Future<Item = (), Error = Error>> {
        let msg = SetValue {
            path: path.to_owned(),
            content: value.to_owned(),
        };
        let fut = self.addr.send(msg)
            .from_err::<Error>()
            .and_then(|x| x.map_err(Error::from));
        Box::new(fut)
    }
}
