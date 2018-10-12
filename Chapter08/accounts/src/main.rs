extern crate clap;
#[macro_use]
extern crate diesel;
extern crate failure;
extern crate serde_derive;

use clap::{
    crate_authors, crate_description, crate_name, crate_version, App, AppSettings, Arg, SubCommand,
};
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use failure::Error;

mod models;
mod schema;

fn create_user(conn: &SqliteConnection, name: &str, email: &str) -> Result<(), Error> {
    let uuid = format!("{}", uuid::Uuid::new_v4());

    let new_user = models::NewUser {
        id: &uuid,
        name: &name,
        email: &email,
    };

    diesel::insert_into(schema::users::table)
        .values(&new_user)
        .execute(conn)?;

    Ok(())
}

struct User {
    name: String,
    email: String,
}

const CMD_ADD: &str = "add";
const CMD_LIST: &str = "list";

fn main() -> Result<(), Error> {

    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .setting(AppSettings::SubcommandRequired)
        .arg(
            Arg::with_name("database")
            .short("d")
            .long("db")
            .value_name("FILE")
            .help("Sets a file name of a database")
            .takes_value(true),
            )
        .subcommand(SubCommand::with_name(CMD_ADD).about("add user to the table"))
        .get_matches();

    let path = matches.value_of("database")
        .unwrap_or("test.db");
    let manager = ConnectionManager::<SqliteConnection>::new(path);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    match matches.subcommand() {
        (CMD_ADD, Some(matches)) => {
            let conn = pool.get()?;
            let name = matches.value_of("name").unwrap();
            let email = matches.value_of("email").unwrap();
            create_user(&conn, name, email)?;
        }
        (CMD_LIST, Some(matches)) => {
            use self::schema::users::dsl::*;
            let conn = pool.get()?;
            let mut items = users
                //.filter(id.eq(&uuid))
                .load::<models::User>(&conn)?;
            for user in items {
                println!("{:?}", user);
            }
        }
        _ => {
            matches.usage(); // but unreachable
        }
    }

    Ok(())
}
