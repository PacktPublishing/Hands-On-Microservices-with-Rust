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

use diesel::{Connection, ExpressionMethods, OptionalExtension, PgConnection, QueryDsl, RunQueryDsl, insert_into};
use chrono::Utc;
use failure::{Error, format_err};
use self::models::{Channel, Id, Membership, Message, User};
use self::schema::{channels, memberships, messages, users};
use std::env;

pub struct Api {
    conn: PgConnection,
}

impl Api {
    pub fn connect() -> Result<Self, Error> {
        let database_url = env::var("DATABASE_URL")
            .unwrap_or("postgres://postgres@localhost:5432".to_string());
        let conn = PgConnection::establish(&database_url)?;
        Ok(Self { conn })
    }

    pub fn register_user(&self, email: &str) -> Result<User, Error> {
        insert_into(users::table)
            .values((
                    users::email.eq(email),
                    ))
            .returning((
                    users::id,
                    users::email
                    ))
            .get_result(&self.conn)
            .map_err(Error::from)
    }

    pub fn create_channel(&self, user_id: Id, title: &str, is_public: bool)
        -> Result<Channel, Error>
    {
        self.conn.transaction::<_, _, _>(|| {
            let channel: Channel = insert_into(channels::table)
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
                        channels::created_at,
                        channels::updated_at,
                        ))
                .get_result(&self.conn)
                .map_err(Error::from)?;
            self.add_member(channel.id, user_id)?;
            Ok(channel)
        })
    }

    pub fn publish_channel(&self, channel_id: Id) -> Result<Channel, Error> {
        use crate::schema::channels::dsl::*;
        let channel = channels
            .filter(id.eq(channel_id))
            .select((id, user_id, title, is_public, created_at, updated_at))
            .first::<Channel>(&self.conn)
            .optional()
            .map_err(Error::from)?;
        if let Some(channel) = channel {
            diesel::update(channels.find(channel_id))
                .set(is_public.eq(true))
                .get_result::<Channel>(&self.conn)
                .map_err(Error::from)
            /*
            diesel::update(channels::table)
                .filter(channels::id.eq(channel.id))
                .set(channels::is_public.eq(true))
                .execute(&self.conn)?;
            */
        } else {
            Err(format_err!("channel not found"))
        }
    }

    pub fn add_member(&self, channel_id: Id, user_id: Id)
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
            .get_result(&self.conn)
            .map_err(Error::from)
    }

    pub fn add_message(&self, channel_id: Id, user_id: Id, text: &str)
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
            .get_result(&self.conn)
            .map_err(Error::from)
    }
}

#[cfg(test)]
mod tests {
    use super::Api;

    #[test]
    fn create_users() {
        let api = Api::connect().unwrap();
        let user_1 = api.register_user("user_1@example.com").unwrap();
        let user_2 = api.register_user("user_2@example.com").unwrap();
        let channel = api.create_channel(user_1.id, "My Channel", false).unwrap();
        api.publish_channel(channel.id).unwrap();
        api.add_member(channel.id, user_2.id).unwrap();
        api.add_message(channel.id, user_2.id, "Hi!").unwrap();
    }
}
