use config::{Config, OutputFormat};
use net::http::tls_client;
use utils::console::*;

use base64;
use hyper::header::{ContentType, Authorization, Basic};
use serde::Deserialize;
use serde_json;
use serde_urlencoded;
use std::io;
use std::io::Read;
use std::str;
use webbrowser;


error_chain! {
}

#[derive(Debug)]
pub struct CliOAuth {
    pub client_id: String,
    pub client_secret: String,
    pub auth_endpoint: String,
pub token_endpoint: String,
    pub redirect_uri: String,
    pub open_browser: bool,
}

impl<'a> CliOAuth {
    // Step 1 -- get code
    pub fn get_code(self, extra_params: &mut Vec<(&str, String)>) -> Result<Code> {
        info("Requesting authentication code ...");
        let mut parameters = vec!(
            ("client_id", self.client_id.clone()),
            ("redirect_uri", self.redirect_uri.clone()),
        );
        parameters.append(extra_params);
        let parameters_enc = serde_urlencoded::to_string(&parameters).chain_err(|| "URL serialization failed")?;

        let auth_url = format!("{}?{}", self.auth_endpoint, parameters_enc);
        if self.open_browser {
            msgln("Please authenticate in the web browser window, wait for the redirect, enter the code into the terminal, and then press return ...");
            webbrowser::open(&auth_url).chain_err(|| "Failed to open web browser")?;
        } else {
            msgln("Please authenticate at the following URL, wait for the redirect, enter the code into the terminal, and then press return ...");
            msgln(format!("\n\t{}\n", auth_url));
        }
        msg("Authentication code: ");
        let mut input = String::new();
        let _ = io::stdin().read_line(&mut input);
        let code = input.trim();

        let code = Code {
            code: code.to_string(),
            client_id: self.client_id,
            client_secret: self.client_secret,
            token_endpoint: self.token_endpoint,
            redirect_uri: self.redirect_uri,
        };

        Ok(code)
    }
}

#[derive(Debug)]
enum GrantType {
    AuthorizationCode,
}

#[derive(Debug)]
pub struct Code {
    pub code: String,
    pub client_id: String,
    pub client_secret: String,
    pub token_endpoint: String,
    pub redirect_uri: String,
}

#[derive(Debug)]
pub struct CodeWithBasicAuthScheme {
    code: Code,
    grant_type: GrantType
}

#[derive(Debug)]
pub struct CodeWithUrlScheme {
    code: Code,
}

/*
 * This trait and its implementation are just here to allow us to implement a method on Result<Code>.
 * This is normally not allowed to we need to this indirection. For this to work, the trait has to be in scope
 * at call site.
 */
pub trait CodeWithBasicAuth {
    fn with_basic_auth(self) -> Result<CodeWithBasicAuthScheme>;
}

impl CodeWithBasicAuth for Result<Code> {
    fn with_basic_auth(self) -> Result<CodeWithBasicAuthScheme> {
        self.map(|code| CodeWithBasicAuthScheme { code: code, grant_type: GrantType::AuthorizationCode })
    }
}

/*
 * This trait and its implementation are just here to allow us to implement a method on Result<Code>.
 * This is normally not allowed to we need to this indirection. For this to work, the trait has to be in scope
 * at call site.
 */
pub trait CodeWithBasicAuthSchemeResult {
    fn exchange_for_token<T: Deserialize>(self, config: &Config) -> Result<T>;
}

impl CodeWithBasicAuthSchemeResult for Result<CodeWithBasicAuthScheme> {
    fn exchange_for_token<T: Deserialize>(self, config: &Config) -> Result<T> {
        self.and_then(|code| code.do_exchange_for_token(config))
    }
}

impl CodeWithBasicAuthScheme {
    // Step 2 -- Exchange authentication code for access token
    #[allow(unused_variables)] // for status codes
    fn do_exchange_for_token<T: Deserialize>(self, config: &Config) -> Result<T> {
        info(format!("Using the authentication code '{}'.", self.code.code));
        info("Requesting authentication token ...");

        let client_credential = format!("{}:{}",
                                        &self.code.client_id, &self.code.client_secret);
        let client_credential_enc = base64::encode(&client_credential.into_bytes()[..]);
        let grant_type = match self.grant_type {
            GrantType::AuthorizationCode => "authorization_code"
        };
        let input = format!("grant_type={}&redirect_uri={}&code={}",
                            grant_type, self.code.redirect_uri, self.code.code);

        let client = tls_client().chain_err(|| "Could not create TLS client")?;
        let mut response = client
            .post(&self.code.token_endpoint)
            .header(Authorization(Basic { username: self.code.client_id, password: Some(self.code.client_secret) }))
            .header(ContentType(mime!(Application / WwwFormUrlEncoded)))
            .body(&input)
            .send()
            .chain_err(|| "Failed to finish HTTP request")?;

        let mut buffer = Vec::new();
        response.read_to_end(&mut buffer).chain_err(|| "Failed to read HTTP response")?;

        if config.general.output_format == OutputFormat::JSON {
            info("Received response: ");
            msgln(str::from_utf8(&buffer).chain_err(|| "Failed to print buffer")?);
        }
        let token: T = serde_json::from_slice(&buffer).chain_err(|| "JSON parsing failed")?;

        Ok(token)
    }
}


// https://tools.ietf.org/html/rfc6749#section-2.3.1
/*
 * This trait and its implementation are just here to allow us to implement a method on Result<Code>.
 * This is normally not allowed to we need to this indirection. For this to work, the trait has to be in scope
 * at call site.
 */
pub trait CodeWithUrl {
    fn with_url(self) -> Result<CodeWithUrlScheme>;
}

impl CodeWithUrl for Result<Code> {
    fn with_url(self) -> Result<CodeWithUrlScheme> {
        self.map(|code| CodeWithUrlScheme { code: code })
    }
}

/*
 * This trait and its implementation are just here to allow us to implement a method on Result<Code>.
 * This is normally not allowed to we need to this indirection. For this to work, the trait has to be in scope
 * at call site.
 */
pub trait CodeWithUrlSchemeResult {
    fn exchange_for_token<T: Deserialize>(self, config: &Config) -> Result<T>;
}

impl CodeWithUrlSchemeResult for Result<CodeWithUrlScheme> {
    fn exchange_for_token<T: Deserialize>(self, config: &Config) -> Result<T> {
        self.and_then(|code| code.do_exchange_for_token(config))
    }
}

impl CodeWithUrlScheme {
    // Step 2 -- Exchange authentication code for access token
    #[allow(unused_variables)] // for status codes
    fn do_exchange_for_token<T: Deserialize>(self, config: &Config) -> Result<T> {
        info("Requesting authentication token ...");

        let parameters = &[
            ("client_id", self.code.client_id),
            ("client_secret", self.code.client_secret),
            ("code", self.code.code.to_string()),
        ];
        let parameters_enc = serde_urlencoded::to_string(&parameters).chain_err(|| "URL serialization failed")?;
        let url = format!("https://slack.com/api/oauth.access?{}", parameters_enc);

        let client = tls_client().chain_err(|| "Could not create TLS client")?;
        let mut response = client
            .get(&url)
            .send()
            .chain_err(|| "Failed to finish HTTP request")?;

        let mut buffer = Vec::new();
        response.read_to_end(&mut buffer).chain_err(|| "Failed to read HTTP response")?;

        if config.general.output_format == OutputFormat::JSON {
            info("Received response: ");
            msgln(str::from_utf8(&buffer).chain_err(|| "Failed to print buffer")?);
        }
        let token: T = serde_json::from_slice(&buffer).chain_err(|| "JSON parsing failed")?;

        Ok(token)
    }
}
