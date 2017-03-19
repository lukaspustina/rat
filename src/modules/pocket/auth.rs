use config::{Config, OutputFormat};
use net::{curl, HttpVerb};
use utils::console::*;

use clap::{App, Arg, ArgMatches, SubCommand};
use serde_json;
use std::io;
use std::str;
use webbrowser;

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
        .arg(Arg::with_name("browser")
            .long("browser")
            .help("Open authentication page in default web browser"))
}

pub fn call(args: Option<&ArgMatches>, config: &Config) -> Result<()> {
    let use_browser = args.ok_or(false).unwrap().is_present("browser");
    auth(config, use_browser).chain_err(|| ErrorKind::PocketAuthFailed)
}

#[allow(unused_variables)] // for status codes
fn auth(config: &Config, use_browser: bool) -> Result<()> {
    let consumer_key = &config.pocket.consumer_key;

    // Step 1 -- get code
    info("Requesting authentication code ...");
    let mut buffer = Vec::new();
    let step_1 = Step1 { consumer_key: consumer_key, redirect_uri: REDIRECT_URI };
    let step_1_json = serde_json::to_string(&step_1).chain_err(|| "JSON serialization failed")?;

    // TODO: Only continue if 200
    let step_1_status_code = curl(
        "https://getpocket.com/v3/oauth/request",
        HttpVerb::POST,
        Some(HEADERS),
        Some(&step_1_json.into_bytes()),
        Some(&mut buffer)
    ).chain_err(|| "Curl failed")?;
    if config.general.output_format == OutputFormat::JSON {
        info("Received response:");
        msgln(str::from_utf8(&buffer).chain_err(|| "Failed to print buffer")?);
    }
    let step_1_result: Step1Result = serde_json::from_slice(&buffer).chain_err(|| "JSON parsing failed")?;

    // Step 2 -- Wait for Web UI authentication
    info("Authorizing code via Pocket ...");
    let auth_url = format!("https://getpocket.com/auth/authorize?request_token={}&redirect_uri={}",
                           step_1_result.code, REDIRECT_URI);
    if use_browser {
        msg("Please authenticate in the web browser window and then press return ...");
        webbrowser::open(&auth_url).chain_err(|| "Failed to open web browser")?;
    } else {
        msgln("Please authenticate at the following URL and then press return ...");
        msgln(format!("\n\t{}\n", auth_url));
    }
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
        Some(HEADERS),
        Some(&step_3_json),
        Some(&mut buffer)
    ).chain_err(|| "Curl failed")?;
    if config.general.output_format == OutputFormat::JSON {
        info("Received response:");
        msgln(str::from_utf8(&buffer).chain_err(|| "Failed to print buffer")?);
    }
    let step_3_result: Step3Result = serde_json::from_slice(&buffer).chain_err(|| "JSON parsing failed")?;

    msgln(format!("Received access token for user '{}'. Please add the following line to your configuration, section '[pocket]'."
                  , step_3_result.username));
    msgln(format!("\naccess_token = '{}'\n", step_3_result.access_token));

    Ok(())
}