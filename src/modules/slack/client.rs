pub use self::auth::auth;

mod auth {
    use config::Config;
    use net::oauth::*;
    use utils::console::*;

    static REDIRECT_URI: &'static str = "https://lukaspustina.github.io/rat/redirects/slack.html";

    error_chain! {
        errors {
           HttpAuthCallFailed {
                description("HTTP authentication call failed")
                display("HTTP authentication call failek")
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

    pub fn auth(config: &Config, open_browser: bool) -> Result<()> {
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
            .chain_err(|| ErrorKind::HttpAuthCallFailed)?;

        msgln(format!("Received access token for user id '{}', team '{}'. Please add the following line to your configuration, section '[slack]'."
                      , token.user_id, token.team_name));
        msgln(format!("\naccess_token = '{}'\n", token.access_token));

        Ok(())
    }
}