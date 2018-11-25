use reqwest::{self, StatusCode};

mod utils;

#[test]
fn mails_healthcheck() {
    utils::healthcheck("http://localhost:8002/", "Mailer Microservice");
}
