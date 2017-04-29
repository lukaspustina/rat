use super::client;
use super::auth::CenterDeviceToken;

use config::{Config, OutputFormat};
use utils::console::*;
use utils::output;

use clap::{App, ArgMatches, SubCommand};
use serde_json;
use std::str;

pub const NAME: &'static str = "refresh_token";

error_chain! {
    errors {
        CenterDeviceRefrehTokenFailed {
            description("failed to refresh access token")
            display("failed to refresh access token")
        }

        OutputFailed {
            description("output failed")
            display("output failed")
        }
    }
}

pub fn build_sub_cli() -> App<'static, 'static> {
    SubCommand::with_name(NAME)
        .about("Re-fresh access token")
}

pub fn call(_: Option<&ArgMatches>, config: &Config) -> Result<()> {
    info(format!("Refreshing access token"));
    let json = client::refresh_token(
        config.centerdevice.refresh_token.as_ref().unwrap(),
        &config.centerdevice.client_id,
        &config.centerdevice.client_secret,
    ).chain_err(|| ErrorKind::CenterDeviceRefrehTokenFailed)?;

    output(&json, &config.general.output_format)
}


fn output(json: &str, format: &OutputFormat) -> Result<()> {
    match *format {
        OutputFormat::HUMAN => output_human(json),
        OutputFormat::JSON => output::as_json(json).chain_err(|| ErrorKind::OutputFailed),
    }
}

fn output_human(json: &str) -> Result<()> {
    let result: CenterDeviceToken = serde_json::from_str(json).chain_err(|| "JSON parsing failed")?;
    let days = result.expires_in / 60 / 60 / 24;
    msgln(format!("Received refreshed access token expiring in {} days. Please change the following line in your configuration, section '[centerdevice]'.",
                  days));
    msgln(format!("\naccess_token = '{}'\n", result.access_token));

    Ok(())
}
