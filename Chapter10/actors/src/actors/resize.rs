use actix::{Actor, Handler, Message, SyncContext};
use image::{ImageResult, FilterType};

type Buffer = Vec<u8>;

pub struct Resize {
    pub buffer: Buffer,
    pub width: u16,
    pub height: u16,
}

impl Message for Resize {
    type Result = ImageResult<Buffer>;
}

pub struct ResizeActor;

impl Actor for ResizeActor {
    type Context = SyncContext<Self>;
}

impl Handler<Resize> for ResizeActor {
    type Result = ImageResult<Buffer>;

    fn handle(&mut self, data: Resize, _: &mut SyncContext<Self>) -> Self::Result {
        let format = image::guess_format(&data.buffer)?;
        let img = image::load_from_memory(&data.buffer)?;
        let scaled = img.resize(data.width as u32, data.height as u32, FilterType::Lanczos3);
        let mut result = Vec::new();
        scaled.write_to(&mut result, format)?;
        Ok(result)
    }
}
