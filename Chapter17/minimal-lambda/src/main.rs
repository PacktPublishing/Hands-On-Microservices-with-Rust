use serde_derive::{Serialize, Deserialize};
use lambda_runtime::{lambda, Context, error::HandlerError};
use rand::Rng;
use rand::distributions::{Bernoulli, Normal, Uniform};
use std::error::Error;
use std::ops::Range;

#[derive(Deserialize)]
#[serde(tag = "distribution", content = "parameters", rename_all = "lowercase")]
enum RngRequest {
    Uniform {
        #[serde(flatten)]
        range: Range<i32>,
    },
    Normal {
        mean: f64,
        std_dev: f64,
    },
    Bernoulli {
        p: f64,
    },
}

#[derive(Serialize)]
struct RngResponse {
    value: f64,
}

fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::init_with_level(log::Level::Debug).unwrap();
    lambda!(rng_handler);
    Ok(())
}

fn rng_handler(event: RngRequest, _ctx: Context) -> Result<RngResponse, HandlerError> {
    let mut rng = rand::thread_rng();
    let value = {
        match event {
            RngRequest::Uniform { range } => {
                rng.sample(Uniform::from(range)) as f64
            },
            RngRequest::Normal { mean, std_dev } => {
                rng.sample(Normal::new(mean, std_dev)) as f64
            },
            RngRequest::Bernoulli { p } => {
                rng.sample(Bernoulli::new(p)) as i8 as f64
            },
        }
    };
    Ok(RngResponse { value })
}
