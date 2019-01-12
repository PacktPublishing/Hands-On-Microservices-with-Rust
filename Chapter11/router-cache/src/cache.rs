use actix::prelude::*;
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

pub struct SetValue {
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

pub struct GetValue {
    pub path: String,
}

impl Message for GetValue {
    type Result = Result<Vec<u8>, RedisError>;
}

impl Handler<GetValue> for CacheActor {
    type Result = Result<Vec<u8>, RedisError>;

    fn handle(&mut self, msg: GetValue, _: &mut Self::Context) -> Self::Result {
        self.client.get(msg.path)
    }
}
