use reqwest::{self, StatusCode};

mod utils;

#[test]
fn router_healthcheck() {
    utils::healthcheck("http://localhost:8000/healthcheck", "Router Microservice");
}
