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

const CMD_CRATE: &str = "create";
const CMD_ADD: &str = "add";

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
        .subcommand(SubCommand::with_name(CMD_ADD).about("add user to the table"))
        .get_matches();

    let addr = matches.value_of("database")
        .unwrap_or("postgres://postgres@localhost:5433");
    let conn = Connection::connect(addr, TlsMode::None)?;

    match matches.subcommand() {
        (CMD_CRATE, _) => {
            create_table(&conn)?;
        }
        (CMD_ADD, Some(matches)) => {
            let name = matches.value_of("name").unwrap();
            let email = matches.value_of("email").unwrap();
            create_user(&conn, name, email)?;
        }
        _ => {
            matches.usage(); // but unreachable
        }
    }

    Ok(())
}
