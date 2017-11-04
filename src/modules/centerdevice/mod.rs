use config::Config;
use errors::*;

use clap::{App, ArgMatches, SubCommand};

pub const NAME: &'static str = "centerdevice";

mod auth;
mod client;
mod collections;
mod delete;
mod download;
mod refresh_token;
mod search;
mod status;
mod upload;

#[derive(Debug, Deserialize)]
pub struct CenterDeviceConfig {
    client_id: String,
    client_secret: String,
    refresh_token: Option<String>,
    access_token: Option<String>,
    #[serde(default = "default_api_base_url")]
    api_base_url: String,
}

fn default_api_base_url() -> String {
    "centerdevice.de".to_string()
}

pub fn build_sub_cli() -> App<'static, 'static> {
    SubCommand::with_name(NAME)
        .about("CenterDevice status etc.")
        .subcommand(auth::build_sub_cli())
        .subcommand(collections::build_sub_cli())
        .subcommand(delete::build_sub_cli())
        .subcommand(download::build_sub_cli())
        .subcommand(refresh_token::build_sub_cli())
        .subcommand(search::build_sub_cli())
        .subcommand(status::build_sub_cli())
        .subcommand(upload::build_sub_cli())
}

pub fn call(cli_args: Option<&ArgMatches>, config: &Config) -> Result<()> {
    let subcommand = cli_args.unwrap();
    let subcommand_name = subcommand.subcommand_name().ok_or_else(|| ErrorKind::NoSubcommandSpecified(NAME.to_string()))?;
    match subcommand_name {
        auth::NAME => auth::call(subcommand.subcommand_matches(subcommand_name), config)
            .chain_err(|| ErrorKind::ModuleFailed(NAME.to_string())),
        collections::NAME => collections::call(subcommand.subcommand_matches(subcommand_name), config)
            .chain_err(|| ErrorKind::ModuleFailed(NAME.to_string())),
        delete::NAME => delete::call(subcommand.subcommand_matches(subcommand_name), config)
            .chain_err(|| ErrorKind::ModuleFailed(NAME.to_string())),
        download::NAME => download::call(subcommand.subcommand_matches(subcommand_name), config)
            .chain_err(|| ErrorKind::ModuleFailed(NAME.to_string())),
        refresh_token::NAME => refresh_token::call(subcommand.subcommand_matches(subcommand_name), config)
            .chain_err(|| ErrorKind::ModuleFailed(NAME.to_string())),
        search::NAME => search::call(subcommand.subcommand_matches(subcommand_name), config)
            .chain_err(|| ErrorKind::ModuleFailed(NAME.to_string())),
        status::NAME => status::call(subcommand.subcommand_matches(subcommand_name), config)
            .chain_err(|| ErrorKind::ModuleFailed(NAME.to_string())),
        upload::NAME => upload::call(subcommand.subcommand_matches(subcommand_name), config)
            .chain_err(|| ErrorKind::ModuleFailed(NAME.to_string())),
        _ => Ok(())
    }
}
