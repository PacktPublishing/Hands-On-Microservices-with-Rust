extern crate config;
extern crate crypto;
#[macro_use]
extern crate diesel;
extern crate env_logger;
extern crate failure;
extern crate log;
extern crate postgres;
#[macro_use]
extern crate rouille;
extern crate serde_derive;

use crypto::pbkdf2::{pbkdf2_check, pbkdf2_simple};
use diesel::prelude::*;
use diesel::dsl::{exists, select};
use diesel::r2d2::ConnectionManager;
use failure::{format_err, Error};
use log::debug;
use postgres::{Connection, TlsMode};
use rouille::{router, Request, Response};
use serde_derive::{Deserialize, Serialize};

type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

mod models;
mod schema;

#[derive(Deserialize)]
struct Config {
    address: Option<String>,
    database: Option<String>,
}

#[derive(Serialize)]
struct UserId {
    id: String,
}

fn main() -> Result<(), Error> {
    env_logger::init();
    let mut config = config::Config::default();
    config.merge(config::Environment::with_prefix("USERS"))?;
    let config: Config = config.try_into()?;
    let bind_address = config.address.unwrap_or("0.0.0.0:8000".into());
    let db_address = config.database.unwrap_or("postgres://localhost/".into());
    let manager = ConnectionManager::<PgConnection>::new(db_address);
    let pool = Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");
    debug!("Starting microservice...");
    rouille::start_server(bind_address, move |request| {
        match handler(&request, &pool) {
            Ok(response) => {
                response
            },
            Err(err) => {
                Response::text(err.to_string())
                    .with_status_code(500)
            }
        }
    })
}

fn handler(request: &Request, pool: &Pool) -> Result<Response, Error> {
    debug!("Request: {:?}", request);
    let resp = router!(request,
        (GET) (/) => {
            Response::text("Users Microservice")
        },
        (POST) (/signup) => {
            let data = post_input!(request, {
                email: String,
                password: String,
            })?;
            debug!("Signup for {}", data.email);
            let user_email = data.email.trim().to_lowercase();
            let user_password = pbkdf2_simple(&data.password, 12345)?;
            {
                use self::schema::users::dsl::*;
                let conn = pool.get()?;
                let user_exists: bool = select(exists(users.filter(email.eq(user_email.clone()))))
                    .get_result(&conn)?;
                if !user_exists {
                    let uuid = uuid::Uuid::new_v4().to_string();
                    let new_user = models::NewUser {
                        id: &uuid,
                        email: &user_email,
                        password: &user_password,
                    };

                    diesel::insert_into(schema::users::table)
                        .values(&new_user)
                        .execute(&conn)?;

                    Response::json(&())
                } else {
                    Response::text(format!("user {} exists", data.email))
                        .with_status_code(400)
                }
            }
        },
        (POST) (/signin) => {
            let data = post_input!(request, {
                email: String,
                password: String,
            })?;
            debug!("Signin for {}", data.email);
            let user_email = data.email.trim().to_lowercase();
            let user_password = data.password;
            {
                use self::schema::users::dsl::*;
                let conn = pool.get()?;
                let user = users.filter(email.eq(user_email))
                    .first::<models::User>(&conn)?;
                let valid = pbkdf2_check(&user_password, &user.password)
                    .map_err(|err| format_err!("pass check error: {}", err))?;
                if valid {
                    let user_id = UserId {
                        id: user.id,
                    };
                    Response::json(&user_id)
                        .with_status_code(200)
                } else {
                    Response::text("access denied")
                        .with_status_code(403)
                }
            }
        },
        _ => {
            Response::empty_404()
        }
    );
    Ok(resp)
}
