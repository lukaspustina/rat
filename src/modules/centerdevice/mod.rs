use clap::{App, ArgMatches, SubCommand};
use config::OutputFormat;

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

pub fn call(cli_args: Option<&ArgMatches>, config: &Config) {
    match cli_args.unwrap().subcommand_name() {
        Some(subcommand) => {
            match subcommand {
                status::NAME => status::call(cli_args.unwrap().subcommand_matches(subcommand), &config),
                _ => {}
            }
        },
        None => {
            println!("No {} command specified. Aborting.", NAME);
            return;
        }
    }
}
