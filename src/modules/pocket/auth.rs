use super::Config;
use net::{curl, HttpVerb};
use utils::console::*;

use clap::{App, ArgMatches, SubCommand};
use serde_json;
use std::io;
use std::str;

pub const NAME: &'static str = "auth";

static HEADERS: &'static [&'static str] = &["X-Accept: application/json", "Content-Type: application/json"];
static REDIRECT_URI: &'static str = "https://lukaspustina.github.io/rat/redirects/pocket.html";

error_chain! {
    errors {
       PocketAuthFailed {
            description("failed to authenticate with Pocket")
            display("failed to authenticate with Pocket")
        }
    }
}

#[derive(Serialize, Debug)]
struct Step1<'a> {
    consumer_key: &'a str,
    redirect_uri: &'a str,
}

#[derive(Deserialize, Debug)]
struct Step1Result {
    code: String,
}

#[derive(Serialize, Debug)]
struct Step3<'a> {
    consumer_key: &'a str,
    code: &'a str,
}

#[derive(Deserialize, Debug)]
struct Step3Result {
    access_token: String,
    username: String,
}

pub fn build_sub_cli() -> App<'static, 'static> {
    SubCommand::with_name(NAME)
        .about("Runs authentication process to generate access token")
}

pub fn call(_: Option<&ArgMatches>, config: &Config) -> Result<()>  {
    auth(config).chain_err(|| ErrorKind::PocketAuthFailed)
}

#[allow(unused_variables)] // for status codes
fn auth(config: &Config) -> Result<()> {
    let consumer_key = &config.consumer_key;

    // Step 1 -- get code
    info("Requesting authentication code ...");
    let mut buffer = Vec::new();
    let step_1 = Step1 { consumer_key: consumer_key, redirect_uri: REDIRECT_URI};
    // TODO: Only continue if 200
    let step_1_json = serde_json::to_string(&step_1).chain_err(|| "JSON serialization failed")?.into_bytes();
    let step_1_status_code = curl(
        "https://getpocket.com/v3/oauth/request",
        HttpVerb::POST,
        Some(&HEADERS),
        Some(&step_1_json),
        Some(&mut buffer)
    ).chain_err(|| "Curl failed")?;
    let step_1_result: Step1Result = serde_json::from_slice(&buffer).chain_err(|| "JSON parsing failed")?;

    // Step 2 -- Wait for Web UI authentication
    info("Authorizing code via Pocket ...");
    msg(format!("Please authenticate at the following URL and then press return ..."));
    msg(format!("\n\thttps://getpocket.com/auth/authorize?request_token={}&redirect_uri={}\n",
             step_1_result.code, REDIRECT_URI));
    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input);

    // Step 3 -- Exchange code for access token
    info("Requesting access token ...");
    let mut buffer = Vec::new();
    let step_3 = Step3 { consumer_key: consumer_key, code: &step_1_result.code };
    let step_3_json = serde_json::to_string(&step_3).chain_err(|| "JSON serialization failed")?.into_bytes();
    // TODO: Only continue if 200
    let step_3_status_code = curl(
        "https://getpocket.com/v3/oauth/authorize",
        HttpVerb::POST,
        Some(&HEADERS),
        Some(&step_3_json),
        Some(&mut buffer)
    ).chain_err(|| "Curl failed")?;
    let step_3_result: Step3Result = serde_json::from_slice(&buffer).chain_err(|| "JSON parsing failed")?;

    msg(format!("Received access token for user '{}'. Please add the following line to your configuration, section '[pocket]'."
             , step_3_result.username));
    msg(format!("\naccess_token = '{}'\n", step_3_result.access_token));

    Ok(())
}