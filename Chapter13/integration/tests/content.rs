use reqwest::{self, StatusCode};

mod utils;

#[test]
fn content_healthcheck() {
    utils::healthcheck("http://localhost:8003/", "Content Microservice");
}
