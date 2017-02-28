use config::OutputFormat;
use errors::*;

use clap::{App, ArgMatches, SubCommand};

pub const NAME: &'static str = "centerdevice";

mod status;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub output_format: OutputFormat,
}

pub fn build_sub_cli() -> App<'static, 'static> {
    SubCommand::with_name(NAME)
        .about("CenterDevice status etc.")
        .subcommand(status::build_sub_cli())
}

pub fn call(cli_args: Option<&ArgMatches>, config: &Config) -> Result<()> {
    let subcommand = cli_args.unwrap();
    let subcommand_name = subcommand.subcommand_name().ok_or(ErrorKind::NoSubcommandSpecified(NAME.to_string()))?;
    match subcommand_name {
        status::NAME => status::call(subcommand.subcommand_matches(subcommand_name), &config)
            .chain_err(|| ErrorKind::ModuleFailed(NAME.to_string())),
        _ => Ok(())
    }
}
