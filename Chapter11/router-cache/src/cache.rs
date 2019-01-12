use actix::prelude::*;
use redis::{Commands, Connection, RedisError};

const CACHE: &str = "cache";

struct CacheActor(Connection);

impl Actor for CacheActor {
    type Context = SyncContext<Self>;
}

struct SetValue {
    path: String,
    content: String,
}

impl Message for SetValue {
    type Result = Result<(), RedisError>;
}

impl Handler<SetValue> for CacheActor {
    type Result = Result<(), RedisError>;

    fn handle(&mut self, msg: SetValue, _: &mut Self::Context) -> Self::Result {
        self.0.hset(CACHE, msg.path, msg.content)
    }
}

struct GetValue {
    path: String,
}

impl Message for GetValue {
    type Result = Result<String, RedisError>;
}

impl Handler<GetValue> for CacheActor {
    type Result = Result<String, RedisError>;

    fn handle(&mut self, msg: GetValue, _: &mut Self::Context) -> Self::Result {
        self.0.hget(CACHE, msg.path)
    }
}
