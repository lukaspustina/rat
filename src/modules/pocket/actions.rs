use super::Config;
use net::{curl, HttpVerb};
use utils::console::*;

use clap::{App, Arg, ArgMatches, SubCommand};
use serde_json;
use serde_urlencoded;
use std::str;

pub const NAME_ARCHIVE: &'static str = "archive";
pub const NAME_READD: &'static str = "readd";
pub const NAME_FAVORITE: &'static str = "favorite";
pub const NAME_UNFAVORITE: &'static str = "unfavorite";
pub const NAME_DELETE: &'static str = "delete";

error_chain! {
    errors {
       PocketActionFailed(action: String) {
            description("action failed to apply to Pocket articles")
            display("action '{}' failed to apply to Pocket article", action)
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Serialize, Debug)]
enum Action {
    archive,
    readd,
    favorite,
    unfavorite ,
    delete,
}

#[allow(non_snake_case)]
#[derive(Serialize, Debug)]
struct ActionRequest<'a> {
    action: &'a str,
    item_id: &'a str,
}

impl<'a> ActionRequest<'a> {
    fn new(action: &'a str, item_id: &'a str) -> Self {
        ActionRequest{ action: action, item_id: item_id }
    }
}

pub fn build_sub_cli() -> Vec<App<'static, 'static>> {
    let mut subcommands: Vec<App<'static, 'static>> = vec![];
    // TODO: Yeah, this screams macro
    subcommands.push(
        SubCommand::with_name(NAME_ARCHIVE)
            .about("Archive saved articles")
            .arg(Arg::with_name("id")
                .index(1)
                .multiple(true)
                .required(true)
                .help("article id")),
    );
    subcommands.push(
        SubCommand::with_name(NAME_READD)
            .about("Re-add (unarchive) an item")
            .arg(Arg::with_name("id")
                .index(1)
                .multiple(true)
                .required(true)
                .help("article id")),
    );
    subcommands.push(
        SubCommand::with_name(NAME_FAVORITE)
            .about("Mark articles as favorite")
            .arg(Arg::with_name("id")
                .index(1)
                .multiple(true)
                .required(true)
                .help("article id")),
    );
    subcommands.push(
        SubCommand::with_name(NAME_UNFAVORITE)
            .about("Remove articles as favorite")
            .arg(Arg::with_name("id")
                .index(1)
                .multiple(true)
                .required(true)
                .help("article id")),
    );
    subcommands.push(
        SubCommand::with_name(NAME_DELETE)
            .about("Delete saved articles")
            .arg(Arg::with_name("id")
            .index(1)
            .multiple(true)
            .required(true)
            .help("article id")),
    );
    subcommands
}

pub fn call(action: &str, args: Option<&ArgMatches>, config: &Config) -> Result<()> {
    let args = args.unwrap();
    let ids = args.values_of("id").unwrap();
    let actions: Vec<ActionRequest> = ids.map(|id| ActionRequest::new(&action, id)).collect();

    send(config, &actions)
        .chain_err(|| ErrorKind::PocketActionFailed(action.to_string()))
}

#[allow(unused_variables)] // for status codes
fn send(config: &Config, actions: &[ActionRequest]) -> Result<()> {
    let mut buffer = Vec::new();
    let actions_json = serde_json::to_string(&actions).chain_err(|| "JSON serialization failed")?;
    let parameters = &[
        ("actions", actions_json),
        ("access_token", config.access_token.as_ref().unwrap().to_string()),
        ("consumer_key", config.consumer_key.to_string())
    ];
    let parameters_enc = serde_urlencoded::to_string(&parameters).chain_err(|| "URL serialization failed")?;

    let url = format!("https://getpocket.com/v3/send?{}", parameters_enc);

    // TODO: Only continue if 200
    let response_status_code = curl(
        &url,
        HttpVerb::GET,
        None,
        None,
        Some(&mut buffer)
    ).chain_err(|| "Curl failed")?;

    let response_str = str::from_utf8(&buffer).chain_err(|| "Data copying failed.")?;
    msg(format!("{}", response_str));

    Ok(())
}
