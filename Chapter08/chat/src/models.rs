use chrono::NaiveDateTime;
use crate::schema::{users, channels, memberships, messages};

pub type Id = i32;

#[derive(Debug, Identifiable, Queryable, Serialize, Deserialize)]
#[table_name = "users"]
pub struct User {
    pub id: Id,
    pub email: String,
}

#[derive(Debug, Insertable, Serialize, Deserialize)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub email: &'a str,
}

#[derive(Debug, Identifiable, Queryable, Serialize, Deserialize)]
#[belongs_to(User)]
#[table_name = "channels"]
pub struct Channel {
    pub id: Id,
    pub user_id: Id,
    pub title: String,
    pub is_public: bool,
}

#[derive(Debug, Insertable, Serialize, Deserialize)]
#[table_name = "channels"]
pub struct NewChannel<'a> {
    pub user_id: Id,
    pub title: &'a str,
    pub is_public: bool,
}

#[derive(Debug, Identifiable, Queryable, Serialize, Deserialize)]
#[belongs_to(Channel)]
#[belongs_to(User)]
#[table_name = "memberships"]
pub struct Membership {
    pub id: Id,
    pub channel_id: Id,
    pub user_id: Id,
}

#[derive(Debug, Insertable, Serialize, Deserialize)]
#[table_name = "memberships"]
pub struct NewMembership {
    pub channel_id: Id,
    pub user_id: Id,
}

#[derive(Debug, Identifiable, Queryable, Serialize, Deserialize)]
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

/*
#[derive(Debug, Insertable, Serialize, Deserialize)]
#[table_name = "messages"]
pub struct NewMessage<'a> {
    pub timestamp: &'a NaiveDateTime,
    pub channel_id: Id,
    pub user_id: Id,
    pub text: &'a str,
}
*/
