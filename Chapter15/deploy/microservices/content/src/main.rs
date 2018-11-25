#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate rocket_contrib;

mod comment;

use comment::{Comment, NewComment};
use diesel::PgConnection;
use rocket::fairing::AdHoc;
use rocket::request::Form;
use rocket_contrib::json::Json;
use rocket_contrib::databases::{database_config, ConfigError};

#[database("postgres_database")]
pub struct Db(PgConnection);

#[post("/new_comment", data = "<comment_form>")]
fn add_new(comment_form: Form<NewComment>, conn: Db) {
    let comment = comment_form.into_inner();
    Comment::insert(comment, &conn);
}

#[get("/list")]
fn index(conn: Db) -> Json<Vec<Comment>> {
    Json(Comment::all(&conn))
}

fn main() -> Result<(), ConfigError> {
    rocket::ignite()
        .attach(Db::fairing())
        .mount("/", routes![index, add_new])
        .launch();
    Ok(())
}
