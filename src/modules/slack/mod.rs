use config::Config;
use errors::*;

use clap::{App, ArgMatches, SubCommand};

pub const NAME: &'static str = "slack";

mod auth;
mod client;

#[derive(Debug, Deserialize)]
pub struct SlackConfig {
    pub client_id: String,
    pub client_secret: String,
    pub access_token: Option<String>,
}

pub fn build_sub_cli() -> App<'static, 'static> {
    SubCommand::with_name(NAME)
        .about("Slack")
        .subcommand(auth::build_sub_cli())
}

pub fn call(cli_args: Option<&ArgMatches>, config: &Config) -> Result<()> {
    let subcommand = cli_args.unwrap();
    let subcommand_name = subcommand.subcommand_name().ok_or_else(|| ErrorKind::NoSubcommandSpecified(NAME.to_string()))?;
    match subcommand_name {
        auth::NAME => auth::call(subcommand.subcommand_matches(subcommand_name), config)
            .chain_err(|| ErrorKind::ModuleFailed(NAME.to_string())),
        _ => Ok(())
    }
}