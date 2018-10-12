#[macro_use]
extern crate diesel;
extern crate failure;
#[macro_use]
extern crate rouille;
extern crate serde_derive;

use diesel::prelude::*;
use diesel::dsl::{exists, select};
use diesel::r2d2::ConnectionManager;
use failure::Error;
use rouille::{router, Request, Response};

type Pool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

mod models;
mod schema;

fn main() {
    let manager = ConnectionManager::<SqliteConnection>::new("test.db");
    let pool = Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");
    rouille::start_server("localhost:8000", move |request| {
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
    let resp = router!(request,
        (GET) (/) => {
            Response::text("Users Microservice")
        },
        (POST) (/signup) => {
            let data = post_input!(request, {
                email: String,
                password: String,
            })?;
            let user_email = data.email.trim().to_lowercase();
            let user_password = data.password;
            {
                use self::schema::users::dsl::*;
                let conn = pool.get()?;
                let user_exists: bool = select(exists(users.filter(email.eq(user_email.clone()))))
                    .get_result(&conn)?;
                if !user_exists {
                    let uuid = format!("{}", uuid::Uuid::new_v4());
                    let new_user = models::NewUser {
                        id: &uuid,
                        email: &user_email,
                        password: &user_password,
                    };

                    diesel::insert_into(schema::users::table)
                        .values(&new_user)
                        .execute(&conn)?;

                    Response::empty_204()
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
            let user_email = data.email;
            let user_password = data.password;
            {
                use self::schema::users::dsl::*;
                let conn = pool.get()?;
                let user = users.filter(email.eq(user_email))
                    .first::<models::User>(&conn)?;
                // You have to use PBKDF2 here
                if user.password == user_password {
                    println!("create JWT token");
                    Response::text("token created")
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
