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
use rocket::request::Form;
use rocket_contrib::json::Json;

#[database("postgres_database")]
pub struct Db(PgConnection);

#[get("/")]
fn index() -> &'static str {
    "Content Microservice"
}

#[post("/new_comment", data = "<comment_form>")]
fn add_new(comment_form: Form<NewComment>, conn: Db) -> Json<()> {
    let comment = comment_form.into_inner();
    Comment::insert(comment, &conn);
    Json(())
}

#[get("/list")]
fn list(conn: Db) -> Json<Vec<Comment>> {
    Json(Comment::all(&conn))
}

fn main() {
    rocket::ignite()
        .attach(Db::fairing())
        .mount("/", routes![index, add_new, list])
        .launch();
}
