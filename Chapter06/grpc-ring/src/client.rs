use failure::Error;
use grpc_ring::Remote;
use std::env;

fn main() -> Result<(), Error> {
    let next = env::var("NEXT")?.parse()?;
    let remote = Remote::new(next)?;
    remote.start_roll_call()?;
    Ok(())
}
