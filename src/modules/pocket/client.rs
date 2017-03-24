pub use self::auth::auth;

mod auth {
    use config::{Config, OutputFormat};
    use utils::console::*;

    use hyper::Client;
    use hyper::header::ContentType;
    use hyper::net::HttpsConnector;
    use hyper_native_tls::NativeTlsClient;
    use mime::Mime;
    use serde_json;
    use std::io;
    use std::io::Read;
    use std::str;
    use webbrowser;

    static REDIRECT_URI: &'static str = "https://lukaspustina.github.io/rat/redirects/pocket.html";

    error_chain! {
        errors {
            HttpAuthCallFailed {
                description("failed to make HTTP auth call")
                display("failed to make HTTP auth call")
            }
        }
    }

    header! { (XAccept, "X-Accept") => [Mime] }

    #[derive(Serialize, Debug)]
    struct CodeRequest<'a> {
        consumer_key: &'a str,
        redirect_uri: &'a str,
    }

    #[derive(Deserialize, Debug)]
    struct Code {
        code: String,
    }

    #[derive(Serialize, Debug)]
    struct TokenRequest<'a> {
        consumer_key: &'a str,
        code: &'a str,
    }

    #[derive(Deserialize, Debug)]
    struct Token {
        access_token: String,
        username: String,
    }

    pub fn auth(config: &Config, open_browser: bool) -> Result<()> {
        do_auth(config, open_browser).chain_err(|| ErrorKind::HttpAuthCallFailed)
    }

    fn do_auth(config: &Config, open_browser: bool) -> Result<()> {
        let ssl = NativeTlsClient::new().chain_err(|| "Failed to create TLS client")?;
        let connector = HttpsConnector::new(ssl);
        let client = Client::with_connector(connector);

        let code = get_code(&client, config)?;
        web_auth(&code, open_browser)?;
        let token = exchange_token(&client, &code, config)?;

        msgln(format!("Received access token for user '{}'. Please add the following line to your configuration, section '[pocket]'."
                      , token.username));
        msgln(format!("\naccess_token = '{}'\n", token.access_token));

        Ok(())
    }

    // Step 1 -- get code
    fn get_code(client: &Client, config: &Config) -> Result<Code> {
        info("Requesting authentication code ...");
        let code_request = CodeRequest { consumer_key: &config.pocket.consumer_key, redirect_uri: REDIRECT_URI };
        let code_request_json = serde_json::to_string(&code_request).chain_err(|| "JSON serialization failed")?;

        let url = "https://getpocket.com/v3/oauth/request";
        let mut response = client
            .post(url)
            .header(XAccept(mime!(Application / Json)))
            .header(ContentType(mime!(Application / Json)))
            .body(&code_request_json)
            .send()
            .chain_err(|| "Failed to send HTTP request")?;

        let mut buffer = Vec::new();
        response.read_to_end(&mut buffer).chain_err(|| "Failed to read HTTP response")?;

        if config.general.output_format == OutputFormat::JSON {
            info("Received response:");
            msgln(str::from_utf8(&buffer).chain_err(|| "Failed to print buffer")?);
        }
        let code: Code = serde_json::from_slice(&buffer).chain_err(|| "JSON parsing failed")?;

        Ok(code)
    }

    // Step 2 -- Wait for Web UI authentication
    fn web_auth(code: &Code, open_browser: bool) -> Result<()> {
        info("Authorizing code via Pocket ...");
        let auth_url = format!("https://getpocket.com/auth/authorize?request_token={}&redirect_uri={}",
                               code.code, REDIRECT_URI);
        if open_browser {
            msg("Please authenticate in the web browser window and then press return ...");
            webbrowser::open(&auth_url).chain_err(|| "Failed to open web browser")?;
        } else {
            msgln("Please authenticate at the following URL and then press return ...");
            msgln(format!("\n\t{}\n", auth_url));
        }
        // Wait for return
        let mut input = String::new();
        let _ = io::stdin().read_line(&mut input);

        Ok(())
    }

    // Step 3 -- Exchange code for access token
    fn exchange_token(client: &Client, code: &Code, config: &Config) -> Result<Token> {
        info("Requesting access token ...");
        let token_request = TokenRequest { consumer_key: &config.pocket.consumer_key, code: &code.code };
        let token_request_token = serde_json::to_string(&token_request).chain_err(|| "JSON serialization failed")?;
        let url = "https://getpocket.com/v3/oauth/authorize";
        let mut response = client
            .post(url)
            .header(XAccept(mime!(Application / Json)))
            .header(ContentType(mime!(Application / Json)))
            .body(&token_request_token)
            .send()
            .chain_err(|| "Failed to send HTTP request")?;

        let mut buffer = Vec::new();
        response.read_to_end(&mut buffer).chain_err(|| "Failed to read HTTP response")?;

        if config.general.output_format == OutputFormat::JSON {
            info("Received response:");
            msgln(str::from_utf8(&buffer).chain_err(|| "Failed to print buffer")?);
        }
        let token: Token = serde_json::from_slice(&buffer).chain_err(|| "JSON parsing failed")?;

        Ok(token)
    }
}