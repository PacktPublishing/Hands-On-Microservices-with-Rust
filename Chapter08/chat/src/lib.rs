#![feature(custom_attribute)]

extern crate chrono;
extern crate clap;
#[macro_use]
extern crate diesel;
extern crate failure;
#[macro_use]
extern crate serde_derive;

mod models;
mod schema;

use diesel::{ExpressionMethods, PgConnection, RunQueryDsl, insert_into};
use chrono::Utc;
use failure::Error;
use self::models::{Channel, Membership, Message, User};
use self::schema::{channels, memberships, messages, users};

pub struct DbApi<'a> {
    conn: &'a PgConnection,
}

impl<'a> DbApi<'a> {
    pub fn register_user(&self, email: String) -> Result<User, Error> {
        insert_into(users::table)
            .values((
                    users::email.eq(email),
                    ))
            .returning((
                    users::id,
                    users::email
                    ))
            .get_result(self.conn)
            .map_err(Error::from)
    }

    pub fn create_channel(&self, user_id: i32, title: String, is_public: bool)
        -> Result<Channel, Error>
    {
        insert_into(channels::table)
            .values((
                    channels::user_id.eq(user_id),
                    channels::title.eq(title),
                    channels::is_public.eq(is_public),
                    ))
            .returning((
                    channels::id,
                    channels::user_id,
                    channels::title,
                    channels::is_public,
                    ))
            .get_result(self.conn)
            .map_err(Error::from)
    }

    pub fn add_member(&self, channel_id: i32, user_id: i32)
        -> Result<Membership, Error>
    {
        insert_into(memberships::table)
            .values((
                    memberships::channel_id.eq(channel_id),
                    memberships::user_id.eq(user_id),
                    ))
            .returning((
                    memberships::id,
                    memberships::channel_id,
                    memberships::user_id,
                    ))
            .get_result(self.conn)
            .map_err(Error::from)
    }

    pub fn add_message(&self, channel_id: i32, user_id: i32, text: String)
        -> Result<Message, Error>
    {
        let timestamp = Utc::now().naive_utc();
        insert_into(messages::table)
            .values((
                    messages::timestamp.eq(timestamp),
                    messages::channel_id.eq(channel_id),
                    messages::user_id.eq(user_id),
                    messages::text.eq(text),
                    ))
            .returning((
                    messages::id,
                    messages::timestamp,
                    messages::channel_id,
                    messages::user_id,
                    messages::text,
                    ))
            .get_result(self.conn)
            .map_err(Error::from)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
