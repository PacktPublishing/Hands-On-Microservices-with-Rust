use serde_derive::Deserialize;
use uuid::Uuid;

mod utils;

const URL: &str = "http://localhost:8001";

fn url(path: &str) -> String {
    URL.to_owned() + path
}

#[test]
fn users_healthcheck() {
    utils::healthcheck(&url("/"), "Users Microservice");
}

#[derive(Deserialize)]
struct UserId {
    id: Uuid,
}

#[test]
fn check_signup_and_signin() {
    let username = utils::rand_str() + "@example.com";
    let password = utils::rand_str();
    let params = vec![
        ("email", username.as_ref()),
        ("password", password.as_ref()),
    ];
    let _: () = utils::post_request(&url("/signup"), params);

    let params = vec![
        ("email", username.as_ref()),
        ("password", password.as_ref()),
    ];
    let _: UserId = utils::post_request(&url("/signin"), params);
}
