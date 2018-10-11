extern crate clap;
extern crate redis;

use clap::{
    crate_authors, crate_description, crate_name, crate_version, App, AppSettings, Arg, SubCommand,
};
use redis::{Client, Commands, Connection, RedisError};

const SESSIONS: &str = "sessions";
const CMD_ADD: &str = "add";
const CMD_REMOVE: &str = "remove";

fn add_session(conn: &Connection, token: &str, uid: &str) -> Result<(), RedisError> {
    conn.hset(SESSIONS, token, uid)
}

fn remove_session(conn: &Connection, token: &str) -> Result<(), RedisError> {
    conn.hdel(SESSIONS, token)
}

fn main() -> Result<(), RedisError> {

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
        .subcommand(SubCommand::with_name(CMD_ADD).about("add a session"))
        .subcommand(SubCommand::with_name(CMD_REMOVE).about("remove a session"))
        .get_matches();

    let addr = matches.value_of("database")
        .unwrap_or("redis://127.0.0.1/");
    let client = Client::open(addr)?;
    let conn = client.get_connection()?;

    match matches.subcommand() {
        (CMD_ADD, Some(matches)) => {
            let token = matches.value_of("token").unwrap();
            let uid = matches.value_of("uid").unwrap();
            add_session(&conn, token, uid)?;
        }
        (CMD_REMOVE, Some(matches)) => {
            let token = matches.value_of("token").unwrap();
            remove_session(&conn, token)?;
        }
        _ => {
            matches.usage(); // but unreachable
        }
    }

    Ok(())
}
