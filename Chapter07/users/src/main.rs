extern crate clap;
extern crate postgres;

use clap::{
    crate_authors, crate_description, crate_name, crate_version, App, AppSettings, Arg, SubCommand,
};
use postgres::{Connection, Error, TlsMode};

fn create_table(conn: &Connection) -> Result<(), Error> {
    conn.execute("CREATE TABLE users (
                    id SERIAL PRIMARY KEY,
                    name VARCHAR NOT NULL,
                    email VARCHAR NOT NULL
                  )", &[])
        .map(drop)
}

fn create_user(conn: &Connection, name: &str, email: &str) -> Result<(), Error> {
    conn.execute("INSERT INTO users (name, email) VALUES ($1, $2)",
                 &[&name, &email])
        .map(drop)
}

fn list_users(conn: &Connection) -> Result<Vec<(String, String)>, Error> {
    let res = conn.query("SELECT name, email FROM users", &[])?.into_iter()
        .map(|row| (row.get(0), row.get(1)))
        .collect();
    Ok(res)
}

const CMD_CRATE: &str = "create";
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
            .value_name("ADDR")
            .help("Sets an address of db connection")
            .takes_value(true),
            )
        .subcommand(SubCommand::with_name(CMD_CRATE).about("create users table"))
        .subcommand(SubCommand::with_name(CMD_ADD).about("add user to the table")
                    .arg(Arg::with_name("NAME")
                         .help("Sets the name of a user")
                         .required(true)
                         .index(1))
                    .arg(Arg::with_name("EMAIL")
                         .help("Sets the email of a user")
                         .required(true)
                         .index(2)))
        .subcommand(SubCommand::with_name(CMD_LIST).about("print list of users"))
        .get_matches();

    let addr = matches.value_of("database")
        .unwrap_or("postgres://postgres@localhost:5432");
    let conn = Connection::connect(addr, TlsMode::None)?;

    match matches.subcommand() {
        (CMD_CRATE, _) => {
            create_table(&conn)?;
        }
        (CMD_ADD, Some(matches)) => {
            let name = matches.value_of("NAME").unwrap();
            let email = matches.value_of("EMAIL").unwrap();
            create_user(&conn, name, email)?;
        }
        (CMD_LIST, _) => {
            let list = list_users(&conn)?;
            for (name, email) in list {
                println!("Name: {:20}    Email: {:20}", name, email);
            }
        }
        _ => {
            matches.usage(); // but unreachable
        }
    }

    Ok(())
}
