mod types;
mod utils;

use self::types::UserId;
use self::utils::{Method, WebApi};

#[test]
fn users_healthcheck() {
    let mut api = WebApi::users();
    api.healthcheck("/", "Users Microservice");
}

#[test]
fn check_signup_and_signin() {
    let mut api = WebApi::users();
    let username = utils::rand_str() + "@example.com";
    let password = utils::rand_str();
    let params = vec![
        ("email", username.as_ref()),
        ("password", password.as_ref()),
    ];
    let _: () = api.request(Method::POST, "/signup", params);

    let params = vec![
        ("email", username.as_ref()),
        ("password", password.as_ref()),
    ];
    let _: UserId = api.request(Method::POST, "/signin", params);
}
