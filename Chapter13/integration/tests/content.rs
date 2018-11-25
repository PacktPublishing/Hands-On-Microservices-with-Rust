mod utils;

use serde_derive::Deserialize;
use self::utils::{content as url, Method};

#[test]
fn content_healthcheck() {
    utils::healthcheck(&url("/"), "Content Microservice");
}

#[derive(Deserialize)]
pub struct Comment {
    pub id: i32,
    pub uid: String,
    pub text: String,
}

#[test]
fn add_comment() {
    let uuid = uuid::Uuid::new_v4().to_string();
    let comment = utils::rand_str();
    let params = vec![
        ("uid", uuid.as_ref()),
        ("text", comment.as_ref()),
    ];
    let _: () = utils::request(Method::POST, &url("/new_comment"), params);

    let comments: Vec<Comment> = utils::request(Method::GET, &url("/list"), vec![]);
    assert!(comments.into_iter().any(|Comment { text, ..}| { text == comment }))
}
