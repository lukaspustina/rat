use config::{Config, OutputFormat};
use net::{curl, HttpVerb};
use utils::console::*;

use clap::{App, Arg, ArgMatches, SubCommand};
use serde_json;
use serde_urlencoded;
use std::io;
use std::str;
use webbrowser;

pub const NAME: &'static str = "auth";

static REDIRECT_URI: &'static str = "https://lukaspustina.github.io/rat/redirects/slack.html";

error_chain! {
    errors {
       SlackAuthFailed {
            description("failed to authenticate with Slack")
            display("failed to authenticate with Slack")
        }
    }
}

#[derive(Deserialize, Debug)]
struct Step2Result {
    ok: bool,
    access_token: String,
    scope: String,
    user_id: String,
    team_name: String,
    team_id: String,
}

pub fn build_sub_cli() -> App<'static, 'static> {
    SubCommand::with_name(NAME)
        .about("Runs authentication process to generate access token")
        .arg(Arg::with_name("browser")
            .long("browser")
            .help("Open authentication page in default web browser"))
}

pub fn call(args: Option<&ArgMatches>, config: &Config) -> Result<()>  {
    let use_browser = args.ok_or(false).unwrap().is_present("browser");
    auth(config, use_browser).chain_err(|| ErrorKind::SlackAuthFailed)
}

#[allow(unused_variables)] // for status codes
fn auth(config: &Config, use_browser: bool) -> Result<()> {
    let client_id = &config.slack.client_id;
    let client_secret = &config.slack.client_secret;

    // Step 1 -- get code
    info("Requesting authentication code ...");
    let parameters = &[
        ("client_id", client_id),
        ("scope", &"channels:read chat:write:user".to_string()),
        ("redirect_uri", &REDIRECT_URI.to_string()),
    ];
    let parameters_enc = serde_urlencoded::to_string(&parameters).chain_err(|| "URL serialization failed")?;

    let auth_url = format!("https://slack.com/oauth/authorize?{}", parameters_enc);
    if use_browser {
        msgln("Please authenticate in the web browser window, wait for the redirect, enter the code into the terminal, and then press return ...");
        webbrowser::open(&auth_url).chain_err(|| "Failed to open web browser")?;
    } else {
        msgln("Please authenticate at the following URL, wait for the redirect, enter the code into the terminal, and then press return ...");
        msgln(format!("\n\t{}\n", auth_url));
    }
    msg("Authentication code: ");
    let mut input = String::new();
    let size = io::stdin().read_line(&mut input);
    let code = input.trim();
    info(format!("Using the authentication code '{}'.", code));

    // Step 2 -- Exchange authentication code for access token
    info("Requesting authentication token ...");
    let parameters = &[
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("code", &code.to_string()),
    ];
    let parameters_enc = serde_urlencoded::to_string(&parameters).chain_err(|| "URL serialization failed")?;
    let url = format!("https://slack.com/api/oauth.access?{}", parameters_enc);

    let mut buffer = Vec::new();
    // TODO: Only continue if 200
    let response_status_code = curl(
        &url,
        HttpVerb::GET,
        None,
        None,
        Some(&mut buffer)
    ).chain_err(|| "Curl failed")?;

    let response_str = str::from_utf8(&buffer).chain_err(|| "Data copying failed.")?;
    let step_2_result: Step2Result = serde_json::from_slice(&buffer).chain_err(|| "JSON parsing failed")?;
    if config.general.output_format == OutputFormat::JSON {
        info("Received response:");
        msgln(str::from_utf8(&buffer).chain_err(|| "Failed to print buffer")?);
    }
    msgln(format!("Received access token for user id '{}', team '{}'. Please add the following line to your configuration, section '[slack]'."
                  , step_2_result.user_id, step_2_result.team_name));
    msgln(format!("\naccess_token = '{}'\n", step_2_result.access_token));

    Ok(())
}

