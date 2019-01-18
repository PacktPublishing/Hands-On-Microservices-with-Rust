use actix::System;
use failure::{format_err, Error};
use image::GenericImageView;
use log::debug;
use queens_rock::Scanner;
use rabbit_actix::queue_actor::{QueueActor, QueueHandler, TaskId};
use rabbit_actix::{QrRequest, QrResponse, REQUESTS, RESPONSES};

struct WokerHandler {}

impl WokerHandler {
    fn scan(&self, data: &[u8]) -> Result<String, Error> {
        let image = image::load_from_memory(data)?;
        let luma = image.to_luma().into_vec();
        let scanner = Scanner::new(
            luma.as_ref(),
            image.width() as usize,
            image.height() as usize,
        );
        scanner
            .scan()
            .extract(0)
            .ok_or_else(|| format_err!("can't extract"))
            .and_then(|code| code.decode().map_err(|_| format_err!("can't decode")))
            .and_then(|data| {
                data.try_string()
                    .map_err(|_| format_err!("can't convert to a string"))
            })
    }
}

impl QueueHandler for WokerHandler {
    type Incoming = QrRequest;
    type Outgoing = QrResponse;

    fn incoming(&self) -> &str {
        REQUESTS
    }
    fn outgoing(&self) -> &str {
        RESPONSES
    }
    fn handle(
        &self,
        _: &TaskId,
        incoming: Self::Incoming,
    ) -> Result<Option<Self::Outgoing>, Error> {
        debug!("In: {:?}", incoming);
        let outgoing = self.scan(&incoming.image).into();
        debug!("Out: {:?}", outgoing);
        Ok(Some(outgoing))
    }
}

fn main() -> Result<(), Error> {
    env_logger::init();
    let mut sys = System::new("rabbit-actix-worker");
    let _ = QueueActor::new(WokerHandler {}, &mut sys)?;
    let _ = sys.run();
    Ok(())
}
