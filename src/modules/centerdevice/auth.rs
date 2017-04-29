use config::Config;
use net::oauth::*;
use utils::console::*;

use clap::{App, Arg, ArgMatches, SubCommand};
use std::str;

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
pub struct CenterDeviceToken {
    pub token_type: String,
    pub expires_in: u32,
    pub refresh_token: String,
    pub access_token: String,
}

pub fn build_sub_cli() -> App<'static, 'static> {
    SubCommand::with_name(NAME)
        .about("Runs authentication process to generate access token")
        .arg(Arg::with_name("browser")
            .long("browser")
            .help("Open authentication page in default web browser"))
}

pub fn call(args: Option<&ArgMatches>, config: &Config) -> Result<()> {
    let open_browser = args.ok_or(false).unwrap().is_present("browser");
    auth(config, open_browser).chain_err(|| ErrorKind::CenterDeviceAuthFailed)
}

fn auth(config: &Config, open_browser: bool) -> Result<()> {
    let oauth = CliOAuth {
        client_id: config.centerdevice.client_id.clone(),
        client_secret: config.centerdevice.client_secret.clone(),
        auth_endpoint: "https://auth.centerdevice.de/authorize".to_string(),
        token_endpoint: "https://auth.centerdevice.de/token".to_string(),
        redirect_uri: REDIRECT_URI.to_string(),
        open_browser: open_browser,
    };

    let token: CenterDeviceToken = oauth
        .get_code(&mut vec!(("response_type", "code".to_string())))
        .with_basic_auth()
        .exchange_for_token(config)
        .chain_err(|| ErrorKind::CenterDeviceAuthFailed)?;

    msgln("Received access and refresh token. Please add the following lines to your configuration, section '[centerdevice]'.");
    msgln(format!("\nrefresh_token = '{}'\naccess_token = '{}'\n", token.refresh_token, token.access_token));

    Ok(())
}
