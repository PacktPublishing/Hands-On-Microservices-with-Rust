mod types;
mod utils;

use self::utils::{Method, WebApi};
use self::types::Comment;

#[test]
fn content_healthcheck() {
    let mut api = WebApi::content();
    api.healthcheck("/", "Content Microservice");
}

#[test]
fn add_comment() {
    let mut api = WebApi::content();
    let uuid = uuid::Uuid::new_v4().to_string();
    let comment = utils::rand_str();
    let params = vec![
        ("uid", uuid.as_ref()),
        ("text", comment.as_ref()),
    ];
    let _: () = api.request(Method::POST, "/new_comment", params);

    let comments: Vec<Comment> = api.request(Method::GET, "/list", vec![]);
    assert!(comments.into_iter().any(|Comment { text, ..}| { text == comment }))
}
