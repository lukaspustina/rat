use config::{Config, OutputFormat};
use net::{curl, HttpVerb};
use utils::console::*;

use base64;
use clap::{App, Arg, ArgMatches, SubCommand};
use serde_json;
use serde_urlencoded;
use std::io;
use std::str;
use webbrowser;

pub const NAME: &'static str = "auth";

static REDIRECT_URI: &'static str = "https://lukaspustina.github.io/rat/redirects/centerdevice.html";

error_chain! {
    errors {
       CenterDeviceAuthFailed {
            description("failed to authenticate with CenterDevice")
            display("failed to authenticate with CenterDevice")
        }
    }
}

#[derive(Deserialize, Debug)]
struct Step2Result {
    token_type: String,
    expires_in: u32,
    refresh_token: String,
    access_token: String,
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
    auth(config, use_browser).chain_err(|| ErrorKind::CenterDeviceAuthFailed)
}

#[allow(unused_variables)] // for status codes
fn auth(config: &Config, use_browser: bool) -> Result<()> {
    let client_id = &config.centerdevice.client_id;
    let client_secret = &config.centerdevice.client_secret;

    // Step 1 -- get code
    info("Requesting authentication code ...");
    let parameters = &[
        ("client_id", client_id),
        ("redirect_uri", &REDIRECT_URI.to_string()),
        ("response_type", &"code".to_string()),
    ];
    let parameters_enc = serde_urlencoded::to_string(&parameters).chain_err(|| "URL serialization failed")?;

    let auth_url = format!("https://auth.centerdevice.de/authorize?{}", parameters_enc);
    if use_browser {
        msgln(format!("Please authenticate in the web browser window, wait for the redirect, enter the code into the terminal, and then press return ..."));
        webbrowser::open(&auth_url).chain_err(|| "Failed to open web browser")?;
    } else {
        msgln(format!("Please authenticate at the following URL, wait for the redirect, enter the code into the terminal, and then press return ..."));
        msgln(format!("\n\t{}\n", auth_url));
    }
    msg("Authentication code: ");
    let mut input = String::new();
    let size = io::stdin().read_line(&mut input);
    let code = input.trim();
    info(format!("Using the authentication code '{}'.", code));

    // Step 2 -- Exchange authentication code for access token
    info("Requesting authentication token ...");

    let client_credential = format!("{}:{}", client_id, client_secret);
    let client_credential_enc = base64::encode(&client_credential.into_bytes()[..]);
    let headers = &[
        &format!("Authorization: Basic {}", client_credential_enc),
        "Content-Type: application/x-www-form-urlencoded"
    ];
    let url = "https://auth.centerdevice.de/token";
    let input = format!("grant_type=authorization_code&redirect_uri={}&code={}", REDIRECT_URI, code)
        .into_bytes();

    let mut buffer = Vec::new();
    // TODO: Only continue if 200
    let response_status_code = curl(
        &url,
        HttpVerb::POST,
        Some(headers),
        Some(&input),
        Some(&mut buffer)
    ).chain_err(|| "Curl failed")?;

    let response_str = str::from_utf8(&buffer).chain_err(|| "Data copying failed.")?;
    if config.general.output_format == OutputFormat::JSON {
        info("Received response:");
        msgln(str::from_utf8(&buffer).chain_err(|| "Failed to print buffer")?);
    }
    let step_2_result: Step2Result = serde_json::from_slice(&buffer).chain_err(|| "JSON parsing failed")?;
    msgln(format!("Received access and refresh token. Please add the following lines to your configuration, section '[centerdevice]'."));
    msgln(format!("\nrefresh_token = '{}'\naccess_token = '{}'\n", step_2_result.refresh_token, step_2_result.access_token));

    Ok(())
}

