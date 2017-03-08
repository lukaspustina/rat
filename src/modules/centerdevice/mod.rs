use config::Config;
use errors::*;

use clap::{App, ArgMatches, SubCommand};

pub const NAME: &'static str = "centerdevice";

mod auth;
mod client;
mod browse_status;
mod status;
mod upload;

#[derive(Debug, Deserialize)]
pub struct CenterDeviceConfig {
    client_id: String,
    client_secret: String,
    refresh_token: Option<String>,
    access_token: Option<String>,
}

pub fn build_sub_cli() -> App<'static, 'static> {
    SubCommand::with_name(NAME)
        .about("CenterDevice status etc.")
        .subcommand(auth::build_sub_cli())
        .subcommand(browse_status::build_sub_cli())
        .subcommand(status::build_sub_cli())
        .subcommand(upload::build_sub_cli())
}

pub fn call(cli_args: Option<&ArgMatches>, config: &Config) -> Result<()> {
    let subcommand = cli_args.unwrap();
    let subcommand_name = subcommand.subcommand_name().ok_or(ErrorKind::NoSubcommandSpecified(NAME.to_string()))?;
    match subcommand_name {
        auth::NAME => auth::call(subcommand.subcommand_matches(subcommand_name), &config)
            .chain_err(|| ErrorKind::ModuleFailed(NAME.to_string())),
        browse_status::NAME => browse_status::call(subcommand.subcommand_matches(subcommand_name), &config)
            .chain_err(|| ErrorKind::ModuleFailed(NAME.to_string())),
        status::NAME => status::call(subcommand.subcommand_matches(subcommand_name), &config)
            .chain_err(|| ErrorKind::ModuleFailed(NAME.to_string())),
        upload::NAME => upload::call(subcommand.subcommand_matches(subcommand_name), &config)
            .chain_err(|| ErrorKind::ModuleFailed(NAME.to_string())),
        _ => Ok(())
    }
}
