use chrono::Utc;
use clap::{App, AppSettings, Arg, SubCommand,
    crate_authors, crate_description, crate_name, crate_version};
use failure::{Error, format_err};
use rusoto_core::Region;
use rusoto_dynamodb::{AttributeValue, DynamoDb, DynamoDbClient,
    QueryInput, UpdateItemInput};
use std::collections::HashMap;

// TODO: add_location
// TODO: list_locations

const CMD_ADD: &str = "add";
const CMD_LIST: &str = "list";

fn main() -> Result<(), Error> {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .setting(AppSettings::SubcommandRequired)
        .arg(
            Arg::with_name("region")
            .long("region")
            .value_name("REGION")
            .help("Sets a region")
            .takes_value(true),
            )
        .arg(
            Arg::with_name("endpoint")
            .long("endpoint-url")
            .value_name("URL")
            .help("Sets an endpoint url")
            .takes_value(true),
            )
        .subcommand(SubCommand::with_name(CMD_ADD).about("add geo record to the table")
                    .arg(Arg::with_name("USER_ID")
                         .help("Sets the id of a user")
                         .required(true)
                         .index(1))
                    .arg(Arg::with_name("LONGITUDE")
                         .help("Sets a longitude of location")
                         .required(true)
                         .index(2))
                    .arg(Arg::with_name("LATITUDE")
                         .help("Sets a latitude of location")
                         .required(true)
                         .index(3)))
        .subcommand(SubCommand::with_name(CMD_LIST).about("print all records for the user")
                    .arg(Arg::with_name("USER_ID")
                         .help("User if to filter records")
                         .required(true)
                         .index(1)))
        .get_matches();

    let region = matches.value_of("endpoint").map(|endpoint| {
        Region::Custom {
            name: "custom".into(),
            endpoint: endpoint.into(),
        }
    }).ok_or_else(|| format_err!("Region not set"))
    .or_else(|_| {
        matches.value_of("region")
            .unwrap_or("us-east-1")
            .parse()
    })?;
    let client = DynamoDbClient::new(region);
    match matches.subcommand() {
        (CMD_ADD, Some(matches)) => {
            let user_id = matches.value_of("USER_ID").unwrap().to_owned();
            let timestamp = Utc::now().to_string();
            let mut key: HashMap<String, AttributeValue> = HashMap::new();
            key.insert("Uid".into(), s_attr(user_id));
            key.insert("TimeStamp".into(), s_attr(timestamp));
            let longitude = matches.value_of("LONGITUDE").unwrap().to_owned();
            let latitude = matches.value_of("LATITUDE").unwrap().to_owned();
            let expression = format!("SET Longitude = :x, Latitude = :y");
            let mut values = HashMap::new();
            values.insert(":x".into(), s_attr(longitude));
            values.insert(":y".into(), s_attr(latitude));
            let update = UpdateItemInput {
                table_name: "Locations".into(),
                key,
                update_expression: Some(expression),
                expression_attribute_values: Some(values),
                ..Default::default()
            };
            client.update_item(update).sync()?;
        }
        (CMD_LIST, Some(matches)) => {
            let user_id = matches.value_of("USER_ID").unwrap().to_owned();
            let expression = format!("Uid = :uid");
            let mut values = HashMap::new();
            values.insert(":uid".into(), s_attr(user_id));
            let query = QueryInput {
                table_name: "Locations".into(),
                key_condition_expression: Some(expression),
                expression_attribute_values: Some(values),
                ..Default::default()
            };
            let items = client.query(query).sync()?
                .items
                .ok_or_else(|| format_err!("No Items"))?;
            for item in items {
                println!("{:?}", item);
            }
        }
        _ => {
            matches.usage(); // but unreachable
        }
    }
    Ok(())
}

fn s_attr(s: String) -> AttributeValue {
    AttributeValue {
        s: Some(s),
        ..Default::default()
    }
}
