use reqwest::{self, StatusCode};

mod utils;

#[test]
fn users_healthcheck() {
    utils::healthcheck("http://localhost:8001/", "Users Microservice");
}
