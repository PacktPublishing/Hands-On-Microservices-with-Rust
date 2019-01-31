use clap::{
    crate_authors, crate_description, crate_name, crate_version, App, AppSettings, Arg, SubCommand,
};
use mysql::{Conn, Error, Opts, OptsBuilder};
use r2d2_mysql::MysqlConnectionManager;
use rayon::prelude::*;
use serde_derive::Deserialize;
use std::io;

fn create_table(conn: &mut Conn) -> Result<(), Error> {
    conn.query("CREATE TABLE users (
                    id INT(6) UNSIGNED AUTO_INCREMENT PRIMARY KEY,
                    name VARCHAR(50) NOT NULL,
                    email VARCHAR(50) NOT NULL
                  )")
        .map(drop)
}

fn create_user(conn: &mut Conn, user: &User) -> Result<(), Error> {
    conn.prep_exec("INSERT INTO users (name, email) VALUES (?, ?)",
                 (&user.name, &user.email))
        .map(drop)
}

fn list_users(conn: &mut Conn) -> Result<Vec<User>, Error> {
    conn.query("SELECT name, email FROM users")?
        .into_iter()
        .try_fold(Vec::new(), |mut vec, row| {
            let row = row?;
            let user = User {
                name: row.get_opt(0).unwrap()?,
                email: row.get_opt(1).unwrap()?,
            };
            vec.push(user);
            Ok(vec)
        })
}

#[derive(Deserialize, Debug)]
struct User {
    name: String,
    email: String,
}

const CMD_CRATE: &str = "create";
const CMD_ADD: &str = "add";
const CMD_LIST: &str = "list";
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
        .subcommand(SubCommand::with_name(CMD_IMPORT).about("import users from csv"))
        .get_matches();

    let addr = matches.value_of("database")
        .unwrap_or("mysql://root:password@localhost:3306/test");
    let opts = Opts::from_url(addr)?;
    let builder = OptsBuilder::from_opts(opts);
    let manager = MysqlConnectionManager::new(builder);
    let pool = r2d2::Pool::new(manager)?;
    let mut conn = pool.get()?;

    match matches.subcommand() {
        (CMD_CRATE, _) => {
            create_table(&mut conn)?;
        }
        (CMD_ADD, Some(matches)) => {
            let name = matches.value_of("NAME").unwrap().to_owned();
            let email = matches.value_of("EMAIL").unwrap().to_owned();
            let user = User { name, email };
            create_user(&mut conn, &user)?;
        }
        (CMD_LIST, _) => {
            let list = list_users(&mut conn)?;
            for user in list {
                println!("Name: {:20}    Email: {:20}", user.name, user.email);
            }
        }
        (CMD_IMPORT, _) => {
            let mut rdr = csv::Reader::from_reader(io::stdin());
            let mut users = Vec::new();
            for user in rdr.deserialize() {
                users.push(user?);
            }
            users.par_iter()
                .map(|user| -> Result<(), failure::Error> {
                    let mut conn = pool.get()?;
                    create_user(&mut conn, &user)?;
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
