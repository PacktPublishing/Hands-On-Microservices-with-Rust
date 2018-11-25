mod utils;

use reqwest::{self, StatusCode};
use self::utils::Method;

const URL: &str = "http://localhost:8002";

fn url(path: &str) -> String {
    URL.to_owned() + path
}

#[test]
fn mails_healthcheck() {
    utils::healthcheck(&url("/"), "Mailer Microservice");
}

#[test]
fn send_mail() {
    let email = utils::rand_str() + "@example.com";
    let code = utils::rand_str();
    let params = vec![
        ("to", email.as_ref()),
        ("code", code.as_ref()),
    ];
    let sent: bool = utils::request(Method::POST, &url("/send"), params);
    assert!(sent);
}
