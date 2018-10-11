extern crate clap;
extern crate csv;
extern crate failure;
extern crate postgres;
extern crate r2d2;
extern crate r2d2_postgres;
extern crate rayon;
extern crate serde_derive;

use clap::{
    crate_authors, crate_description, crate_name, crate_version, App, AppSettings, Arg, SubCommand,
};
use postgres::{Connection, Error};
use r2d2_postgres::{TlsMode, PostgresConnectionManager};
use rayon::prelude::*;
use serde_derive::Deserialize;
use std::io;

fn create_table(conn: &Connection) -> Result<(), Error> {
    conn.execute("CREATE TABLE users (
                    id SERIAL PRIMARY KEY,
                    name VARCHAR NOT NULL,
                    email VARCHAR NOT NULL
                  )", &[])
        .map(drop)
}

fn create_user(conn: &Connection, user: &User) -> Result<(), Error> {
    conn.execute("INSERT INTO users (name, email) VALUES ($1, $2)",
                 &[&user.name, &user.email])
        .map(drop)
}

#[derive(Deserialize)]
struct User {
    name: String,
    email: String,
}

const CMD_CRATE: &str = "create";
const CMD_ADD: &str = "add";
const CMD_IMPORT: &str = "import";

fn main() -> Result<(), failure::Error> {

    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .setting(AppSettings::SubcommandRequired)
        .arg(
            Arg::with_name("database")
            .short("d")
            .long("db")
            .value_name("ADDR")
            .help("Sets an address of db connection")
            .takes_value(true),
            )
        .subcommand(SubCommand::with_name(CMD_CRATE).about("create users table"))
        .subcommand(SubCommand::with_name(CMD_ADD).about("add user to the table"))
        .subcommand(SubCommand::with_name(CMD_IMPORT).about("import users from csv"))
        .get_matches();

    let addr = matches.value_of("database")
        .unwrap_or("postgres://postgres@localhost:5433");
    let manager = PostgresConnectionManager::new(addr, TlsMode::None)?;
    let pool = r2d2::Pool::new(manager)?;
    let conn = pool.get()?;

    match matches.subcommand() {
        (CMD_CRATE, _) => {
            create_table(&conn)?;
        }
        (CMD_ADD, Some(matches)) => {
            let name = matches.value_of("name").unwrap().to_owned();
            let email = matches.value_of("email").unwrap().to_owned();
            let user = User { name, email };
            create_user(&conn, &user)?;
        }
        (CMD_IMPORT, _) => {
            let mut rdr = csv::Reader::from_reader(io::stdin());
            let mut users = Vec::new();
            for user in rdr.deserialize() {
                users.push(user?);
            }
            users.par_iter()
                .map(|user| -> Result<(), failure::Error> {
                    let conn = pool.get()?;
                    create_user(&conn, &user)?;
                    Ok(())
                })
                .for_each(drop);
        }
        _ => {
            matches.usage(); // but unreachable
        }
    }

    Ok(())
}
