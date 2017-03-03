use errors::*;

use clap::{App, ArgMatches, SubCommand};

pub const NAME: &'static str = "pocket";

mod auth;
mod list;
mod search;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub consumer_key: String,
    pub access_token: Option<String>,
}

pub fn build_sub_cli() -> App<'static, 'static> {
    SubCommand::with_name(NAME)
        .about("Pocket: When you find something you want to view later, put it in Pocket.")
        .subcommand(auth::build_sub_cli())
        .subcommand(list::build_sub_cli())
        .subcommand(search::build_sub_cli())
}

pub fn call(cli_args: Option<&ArgMatches>, config: &Config) -> Result<()> {
    let subcommand = cli_args.unwrap();
    let subcommand_name = subcommand.subcommand_name().ok_or(ErrorKind::NoSubcommandSpecified(NAME.to_string()))?;
    match subcommand_name {
        auth::NAME => auth::call(subcommand.subcommand_matches(subcommand_name), &config)
            .chain_err(|| ErrorKind::ModuleFailed(NAME.to_string())),
        list::NAME => list::call(subcommand.subcommand_matches(subcommand_name), &config)
            .chain_err(|| ErrorKind::ModuleFailed(NAME.to_string())),
        search::NAME => search::call(subcommand.subcommand_matches(subcommand_name), &config)
            .chain_err(|| ErrorKind::ModuleFailed(NAME.to_string())),
        _ => Ok(())
    }
}
