use chrono::NaiveDateTime;
use crate::schema::{users, channels, memberships, messages};
use serde_derive::{Deserialize, Serialize};

pub type Id = i32;

#[derive(Debug, Identifiable, Queryable, Serialize, Deserialize)]
#[table_name = "users"]
pub struct User {
    pub id: Id,
    pub email: String,
}

#[derive(Debug, Identifiable, Queryable, Associations, Serialize, Deserialize)]
#[belongs_to(User)]
#[table_name = "channels"]
pub struct Channel {
    pub id: Id,
    pub user_id: Id,
    pub title: String,
    pub is_public: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Identifiable, Queryable, Associations, Serialize, Deserialize)]
#[belongs_to(Channel)]
#[belongs_to(User)]
#[table_name = "memberships"]
pub struct Membership {
    pub id: Id,
    pub channel_id: Id,
    pub user_id: Id,
}

#[derive(Debug, Identifiable, Queryable, Associations, Serialize, Deserialize)]
#[belongs_to(Channel)]
#[belongs_to(User)]
#[table_name = "messages"]
pub struct Message {
    pub id: Id,
    pub timestamp: NaiveDateTime,
    pub channel_id: Id,
    pub user_id: Id,
    pub text: String,
}
