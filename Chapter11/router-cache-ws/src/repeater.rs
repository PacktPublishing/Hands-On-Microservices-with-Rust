use actix::{Actor, Context, Handler, Message, Recipient};
use std::collections::HashSet;
use super::NewComment;

pub struct RepeaterActor {
    listeners: HashSet<Recipient<RepeaterUpdate>>,
}

impl RepeaterActor {
    pub fn new() -> Self {
        Self {
            listeners: HashSet::new(),
        }
    }
}

impl Actor for RepeaterActor {
    type Context = Context<Self>;
}

#[derive(Clone)]
pub struct RepeaterUpdate(pub NewComment);

impl Message for RepeaterUpdate {
    type Result = ();
}

impl Handler<RepeaterUpdate> for RepeaterActor {
    type Result = ();

    fn handle(&mut self, msg: RepeaterUpdate, _: &mut Self::Context) -> Self::Result {
        for listener in &self.listeners {
            listener.do_send(msg.clone()).ok();
        }
    }
}

pub enum RepeaterControl {
    Subscribe(Recipient<RepeaterUpdate>),
    Unsubscribe(Recipient<RepeaterUpdate>),
}

impl Message for RepeaterControl {
    type Result = ();
}

impl Handler<RepeaterControl> for RepeaterActor {
    type Result = ();

    fn handle(&mut self, msg: RepeaterControl, _: &mut Self::Context) -> Self::Result {
        match msg {
            RepeaterControl::Subscribe(listener) => {
                self.listeners.insert(listener);
            }
            RepeaterControl::Unsubscribe(listener) => {
                self.listeners.remove(&listener);
            }
        }
    }
}
