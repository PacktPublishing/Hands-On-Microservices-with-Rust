mod types;
mod utils;

use self::utils::{Method, StatusCode, WebApi};
use self::types::Comment;

#[test]
fn router_healthcheck() {
    let mut api = WebApi::router();
    api.healthcheck("/healthcheck", "Router Microservice");
}

#[test]
fn check_router_full() {
    let mut api = WebApi::router();
    let username = utils::rand_str() + "@example.com";
    let password = utils::rand_str();
    let params = vec![
        ("email", username.as_ref()),
        ("password", password.as_ref()),
    ];
    api.check_status(Method::POST, "/api/signup", params, StatusCode::FOUND);

    let params = vec![
        ("email", username.as_ref()),
        ("password", password.as_ref()),
    ];
    api.check_status(Method::POST, "/api/signin", params, StatusCode::FOUND);

    let comment = utils::rand_str();
    let params = vec![
        ("text", comment.as_ref()),
    ];
    api.check_status(Method::POST, "/api/new_comment", params, StatusCode::FOUND);

    let comments: Vec<Comment> = api.request(Method::GET, "/api/comments", vec![]);
    assert!(comments.into_iter().any(|Comment { text, ..}| { text == comment }))
}
