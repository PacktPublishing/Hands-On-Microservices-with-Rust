use lambda_runtime::{error::HandlerError, lambda, Context};
use log::debug;
use serde_derive::{Serialize, Deserialize};
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::init_with_level(log::Level::Debug)?;
    debug!("TEST");
    lambda!(handler);
    Ok(())
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Location {
    latitude: f64,
    longitude: f64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Request {
    body: RequestBody,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RequestBody {
    pickup_location: Location,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct Unicorn {
    name: String,
    color: String,
    gender: String,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct ResponseBody {
    ride_id: String,
    unicorn: Unicorn,
    unicorn_name: String,
    eta: String,
    rider: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Response {
    body: String,
    status_code: u16,
    headers: HashMap<String, String>,
}

fn handler(event: Value, _: Context) -> Result<Response, HandlerError> {
    debug!("EVENT: {:?}", event);
    //let request = serde_json::from_value(&event).unwrap();
    let unicorn = Unicorn {
        name: "Bucephalus".into(),
        color: "Golden".into(),
        gender: "Male".into(),
    };
    let body = ResponseBody {
        ride_id: "ride-unique-id".into(),
        unicorn,
        unicorn_name: "Bucephalus".into(),
        eta: "30 seconds".into(),
        rider: "congnito-here".into(),
    };
    let mut headers = HashMap::new();
    headers.insert("Access-Control-Allow-Origin".into(), "*".into());
    let body = serde_json::to_string(&body).unwrap();
    let resp = Response {
        status_code: 201,
        body,
        headers,
    };
    Ok(resp)
}

