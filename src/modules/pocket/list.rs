use super::Config;
use net::{curl, HttpVerb};

use clap::{App, ArgMatches, SubCommand};
use serde_json;
use std::str;

pub const NAME: &'static str = "list";

static HEADERS: &'static [&'static str] = &["Content-Type: application/json"];

error_chain! {
    errors {
       PocketListFailed {
            description("failed to list Pocket articles")
            display("failed to list Pocket articles")
        }
    }
}

#[derive(Serialize, Debug)]
struct Request<'a> {
    consumer_key: &'a str,
    access_token: &'a str,
}

pub fn build_sub_cli() -> App<'static, 'static> {
    SubCommand::with_name(NAME)
        .about("List saved articles")
}

pub fn call(_: Option<&ArgMatches>, config: &Config) -> Result<()> {
    list(config).chain_err(|| ErrorKind::PocketListFailed)
}

#[allow(unused_variables)] // for status codes
fn list(config: &Config) -> Result<()> {

    let mut buffer = Vec::new();
    let request = Request{ consumer_key: &config.consumer_key, access_token: &config.access_token.as_ref().unwrap() };
    // TODO: Only continue if 200
    let request_json = &serde_json::to_string(&request).chain_err(|| "JSON serialization failed")?.into_bytes();
    let response_status_code = curl(
        "https://getpocket.com/v3/get",
        HttpVerb::POST,
        Some(&HEADERS),
        Some(request_json),
        Some(&mut buffer)
    ).chain_err(|| "Curl failed")?;

    let response_str = str::from_utf8(&buffer).chain_err(|| "Data copying failed.")?;
    println!("{}", response_str);

    Ok(())
}
