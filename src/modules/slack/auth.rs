use config::Config;
use net::oauth::*;
use utils::console::*;

use clap::{App, Arg, ArgMatches, SubCommand};

pub const NAME: &'static str = "auth";

static REDIRECT_URI: &'static str = "https://lukaspustina.github.io/rat/redirects/slack.html";

error_chain! {
    errors {
       SlackAuthFailed {
            description("failed to authenticate with Slack")
            display("failed to authenticate with Slack")
        }
    }
}

#[derive(Deserialize, Debug)]
struct SlackToken {
    ok: bool,
    access_token: String,
    scope: String,
    user_id: String,
    team_name: String,
    team_id: String,
}

pub fn build_sub_cli() -> App<'static, 'static> {
    SubCommand::with_name(NAME)
        .about("Runs authentication process to generate access token")
        .arg(Arg::with_name("browser")
            .long("browser")
            .help("Open authentication page in default web browser"))
}

pub fn call(args: Option<&ArgMatches>, config: &Config) -> Result<()>  {
    let use_browser = args.ok_or(false).unwrap().is_present("browser");
    auth(config, use_browser).chain_err(|| ErrorKind::SlackAuthFailed)
}

fn auth(config: &Config, open_browser: bool) -> Result<()> {
    let oauth = CliOAuth {
        client_id: config.slack.client_id.clone(),
        client_secret: config.slack.client_secret.clone(),
        auth_endpoint: "https://slack.com/oauth/authorize".to_string(),
        token_endpoint: "https://slack.com/api/oauth.accessn".to_string(),
        redirect_uri: REDIRECT_URI.to_string(),
        open_browser: open_browser,
    };

    let token: SlackToken = oauth
        .get_code(&mut vec!(("scope", "channels:read chat:write:user".to_string())))
        .with_url()
        .exchange_for_token(config)
        .chain_err(|| ErrorKind::SlackAuthFailed)?;

    msgln(format!("Received access token for user id '{}', team '{}'. Please add the following line to your configuration, section '[slack]'."
                  , token.user_id, token.team_name));
    msgln(format!("\naccess_token = '{}'\n", token.access_token));

    Ok(())
}

