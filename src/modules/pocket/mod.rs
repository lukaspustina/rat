use clap::{App, ArgMatches, SubCommand};

pub const NAME: &'static str = "pocket";

mod auth;
mod list;

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
}

pub fn call(cli_args: Option<&ArgMatches>, config: &Config) {
    match cli_args.unwrap().subcommand_name() {
        Some(subcommand) => {
            match subcommand {
                auth::NAME => auth::call(cli_args.unwrap().subcommand_matches(subcommand), &config),
                list::NAME => list::call(cli_args.unwrap().subcommand_matches(subcommand), &config),
                _ => {}
            }
        },
        None => {
            println!("No {} command specified. Aborting.", NAME);
            return;
        }
    }
}
