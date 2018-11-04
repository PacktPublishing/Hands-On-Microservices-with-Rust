#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate rocket_contrib;

mod comment;

use rocket::request::Form;
use rocket_contrib::json::Json;
use diesel::SqliteConnection;

use comment::{Comment, NewComment};

#[database("sqlite_database")]
pub struct Db(SqliteConnection);

#[post("/new_comment", data = "<comment_form>")]
fn add_new(comment_form: Form<NewComment>, conn: Db) -> &'static str {
    let comment = comment_form.into_inner();
    Comment::insert(comment, &conn);
    "Hello, world!"
}

#[post("/list")]
fn index(conn: Db) -> Json<Vec<Comment>> {
    Json(Comment::all(&conn))
}

fn main() {
    rocket::ignite().mount("/", routes![index, add_new]).launch();
}
