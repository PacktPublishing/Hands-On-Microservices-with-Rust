use chrono::Utc;
use lambda_runtime::{error::HandlerError, lambda, Context};
use log::debug;
use rand::thread_rng;
use rand::seq::IteratorRandom;
use rusoto_core::Region;
use rusoto_dynamodb::{AttributeValue, DynamoDb, DynamoDbClient, PutItemError, PutItemInput, PutItemOutput};
use serde_derive::{Serialize, Deserialize};
use std::collections::HashMap;
use std::error::Error;
use uuid::Uuid;

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
    body: String,
    request_context: RequestContext,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RequestContext {
    authorizer: Authorizer,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Authorizer {
    claims: HashMap<String, String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RequestBody {
    pickup_location: Location,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
struct Unicorn {
    name: String,
    color: String,
    gender: String,
}

impl Unicorn {
    fn new(name: &str, color: &str, gender: &str) -> Self {
        Unicorn {
            name: name.to_owned(),
            color: color.to_owned(),
            gender: gender.to_owned(),
        }
    }
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

fn find_unicorn(location: &Location) -> Unicorn {
    debug!("Finding unicorn for {}, {}", location.latitude, location.longitude);
    let unicorns = [
        Unicorn::new("Bucephalus", "Golden", "Male"),
        Unicorn::new("Shadowfax", "White", "Male"),
        Unicorn::new("Rocinante", "Yellow", "Female"),
    ];
    let mut rng = thread_rng();
    unicorns.iter().choose(&mut rng).cloned().unwrap()
}

fn s_attr<T: AsRef<str>>(s: T) -> AttributeValue {
    AttributeValue {
        s: Some(s.as_ref().to_owned()),
        ..Default::default()
    }
}

fn unicorn_map(unicorn: &Unicorn) -> AttributeValue {
    let mut item = HashMap::new();
    item.insert("Name".into(), s_attr(&unicorn.name));
    item.insert("Color".into(), s_attr(&unicorn.color));
    item.insert("Gender".into(), s_attr(&unicorn.gender));
    AttributeValue {
        m: Some(item),
        ..Default::default()
    }
}

fn record_ride(
    conn: &DynamoDbClient,
    ride_id: &str,
    username: &str,
    unicorn: &Unicorn,
) -> Result<PutItemOutput, PutItemError> {
    let mut item: HashMap<String, AttributeValue> = HashMap::new();
    item.insert("RideId".into(), s_attr(ride_id));
    item.insert("User".into(), s_attr(username));
    item.insert("UnicornName".into(), s_attr(&unicorn.name));
    let timestamp = Utc::now().to_string();
    item.insert("RequestTime".into(), s_attr(&timestamp));
    item.insert("Unicorn".into(), unicorn_map(unicorn));
    let put = PutItemInput {
        table_name: "Rides".into(),
        item,
        ..Default::default()
    };
    conn.put_item(put).sync()
}

fn handler(event: Request, _: Context) -> Result<Response, HandlerError> {
    let region = Region::default();
    let client = DynamoDbClient::new(region);
    let username = event
        .request_context
        .authorizer
        .claims
        .get("cognito:username")
        .unwrap()
        .to_owned();
    debug!("USERNAME: {}", username);
    let ride_id = Uuid::new_v4().to_string();
    let request: RequestBody = serde_json::from_str(&event.body).unwrap();
    let unicorn = find_unicorn(&request.pickup_location);
    record_ride(&client, &ride_id, &username, &unicorn).unwrap();
    let body = ResponseBody {
        ride_id: ride_id.clone(),
        unicorn_name: unicorn.name.clone(),
        unicorn,
        eta: "30 seconds".into(),
        rider: username.clone(),
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
