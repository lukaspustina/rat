use super::client;
use config::Config;

use clap::{App, Arg, ArgMatches, SubCommand};

pub const NAME: &'static str = "auth";

error_chain! {
    errors {
       SlackAuthFailed {
            description("failed to authenticate with Slack")
            display("failed to authenticate with Slack")
        }
    }
}

pub fn build_sub_cli() -> App<'static, 'static> {
    SubCommand::with_name(NAME)
        .about("Runs authentication process to generate access token")
        .arg(Arg::with_name("browser")
            .long("browser")
            .help("Open authentication page in default web browser"))
}

pub fn call(args: Option<&ArgMatches>, config: &Config) -> Result<()>  {
    let open_browser = args.ok_or(false).unwrap().is_present("browser");
    client::auth(config, open_browser).chain_err(|| ErrorKind::SlackAuthFailed)
}
