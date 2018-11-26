#![allow(dead_code)]

use serde_derive::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct UserId {
    id: Uuid,
}

#[derive(Deserialize)]
pub struct Comment {
    pub id: i32,
    pub uid: String,
    pub text: String,
}

