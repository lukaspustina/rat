pub use self::auth::auth;
pub use self::list::list;
pub use self::send::send;

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

pub mod list {
    use config::Config;
    use utils::console::*;

    use chrono::{DateTime, NaiveDateTime, UTC};
    use hyper::Client;
    use hyper::header::ContentType;
    use hyper::net::HttpsConnector;
    use hyper_native_tls::NativeTlsClient;
    use serde_json;
    use std::collections::HashMap;
    use std::io::Read;
    use std::str;
    use std::time::Duration;

    error_chain! {
        errors {
           HttpListCallFailed {
                description("HTTP call to list articles failed")
                display("HTTP call to list articles failed")
            }
        }
    }

    #[allow(non_camel_case_types)]
    #[derive(Serialize, Debug)]
    pub enum State {
        unread,
        archive,
        all,
    }

    impl<'a> From<&'a str> for State {
        fn from(s: &'a str) -> Self {
            match s {
                "archive" => State::archive,
                "all" => State::all,
                _ => State::unread,
            }
        }
    }

    #[allow(non_camel_case_types)]
    #[derive(Serialize, Debug)]
    pub enum Sort {
        newest,
        oldest,
        title,
        site,
    }

    impl<'a> From<&'a str> for Sort {
        fn from(s: &'a str) -> Self {
            match s {
                "oldest" => Sort::oldest,
                "title" => Sort::title,
                "site" => Sort::site,
                _ => Sort::newest
            }
        }
    }

    #[allow(non_camel_case_types)]
    #[derive(Serialize, Debug)]
    pub enum DetailType {
        simple,
        complete,
    }

    impl From<bool> for DetailType {
        fn from(b: bool) -> Self {
            if b {
                DetailType::complete
            } else {
                DetailType::simple
            }
        }
    }

    #[allow(non_snake_case)]
    #[derive(Serialize, Debug)]
    pub struct Request<'a> {
        pub consumer_key: &'a str,
        pub access_token: &'a str,
        #[serde(skip_serializing_if = "Option::is_none")] pub state: Option<State>,
        #[serde(skip_serializing_if = "Option::is_none")] pub tag: Option<&'a str>,
        #[serde(skip_serializing_if = "Option::is_none")] pub sort: Option<Sort>,
        pub detailType: DetailType,
        #[serde(skip_serializing_if = "Option::is_none")] pub search: Option<&'a str>,
    }

    pub fn list(config: &Config, request: &Request, since: Option<Duration>, until: Option<Duration>)
                -> Result<String> {
        info("Getting list of your articles ...");
        let mut json = do_list(config, request).chain_err(|| ErrorKind::HttpListCallFailed)?;

        if since.is_some() || until.is_some() {
            let list: ListResult = serde_json::from_str(&json).chain_err(|| "JSON parsing failed")?;
            info(format!("Filtering list of your {} article(s) ...", list.list.len()));
            let list = list.filter(&since, &until);
            json = serde_json::to_string(&list).chain_err(|| "JSON serialization failed")?;
        }

        Ok(json)
    }

    #[allow(unused_variables)] // for status codes
    fn do_list(config: &Config, request: &Request) -> Result<String> {
        let request_json = serde_json::to_string(&request).chain_err(|| "JSON serialization failed")?;
        verboseln(format!("request = {}", request_json));

        let ssl = NativeTlsClient::new().chain_err(|| "Failed to create TLS client")?;
        let connector = HttpsConnector::new(ssl);
        let client = Client::with_connector(connector);

        let url = "https://getpocket.com/v3/get";
        let mut response = client
            .post(url)
            .header(ContentType(mime!(Application / Json)))
            .body(&request_json)
            .send()
            .chain_err(|| "Failed to finish HTTP request")?;

        let mut buffer = Vec::new();
        response.read_to_end(&mut buffer).chain_err(|| "Failed to read HTTP response")?;
        let json = str::from_utf8(&buffer).chain_err(|| "Data copying failed.")?;

        Ok(json.to_string())
    }


    #[derive(Serialize, Deserialize, Debug)]
    pub struct ListResult {
        pub status: i32,
        pub complete: i32,
        pub list: HashMap<String, Article>,
    }

    impl ListResult {
        fn filter(self, since: &Option<Duration>, until: &Option<Duration>) -> Self {
            let mut new_list: HashMap<String, Article> = HashMap::new();
            for (k, v) in self.list {
                if let Some(since) = *since {
                    if v.time_added().unwrap() < since { continue };
                }
                if let Some(until) = *until {
                    if v.time_added().unwrap() > until { continue };
                }
                new_list.insert(k, v);
            }

            ListResult { status: self.status, complete: self.complete, list: new_list }
        }
    }

    #[derive(Debug)]
    pub struct HumanOutput {
        pub id: bool,
        pub title: bool,
        pub url: bool,
        pub t_added: bool,
    }

    impl<'a> From<Vec<&'a str>> for HumanOutput {
        fn from(v: Vec<&'a str>) -> Self {
            let id = v.contains(&"id");
            let title = v.contains(&"title");
            let url = v.contains(&"url");
            let t_added = v.contains(&"t_added");

            HumanOutput { id: id, title: title, url: url, t_added: t_added }
        }
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Article {
        pub item_id: String,
        pub resolved_title: String,
        pub resolved_url: String,
        pub time_added: String,
        pub time_updated: String,
    }

    impl Article {
        fn time_added(&self) -> Result<Duration> {
            let secs: u64 = self.time_added.parse().chain_err(|| "Failed to parse time")?;
            Ok(Duration::from_secs(secs))
        }

        pub fn human_display(&self, human_output: &HumanOutput) -> Result<String> {
            let mut outputs = Vec::new();
            if human_output.id {
                outputs.push(self.item_id.clone());
            }
            if human_output.title {
                outputs.push(format!("'{}'", &self.resolved_title));
            }
            if human_output.url {
                outputs.push(self.resolved_url.clone());
            }
            if human_output.t_added {
                let d = self.time_added()?;
                let dt = DateTime::<UTC>::from_utc(
                    NaiveDateTime::from_timestamp(d.as_secs() as i64, d.subsec_nanos()), UTC);
                outputs.push(format!("added {}", &dt.to_rfc3339()));
            }

            let mut out_str: String = outputs.first().unwrap().to_string();
            outputs.remove(0);
            for o in outputs {
                out_str = out_str + &format!(", {}", o);
            }

            Ok(out_str)
        }
    }
}

pub mod send {
    use config::Config;

    use hyper::Client;
    use hyper::net::HttpsConnector;
    use hyper_native_tls::NativeTlsClient;
    use serde_json;
    use serde_urlencoded;
    use std::io::Read;
    use std::str;

    error_chain! {
        errors {
           HttpActionCallFailed {
                description("HTTP call for action failed")
                display("HTTP call for action failed")
           }
        }
    }

    #[allow(non_camel_case_types)]
    #[derive(Serialize, Debug)]
    // TODO: Use enum instead of strings in ActionRequest
    #[allow(dead_code)]
    enum Action {
        archive,
        read,
        favorite,
        unfavorite,
        delete,
    }

    #[allow(non_snake_case)]
    #[derive(Serialize, Debug)]
    pub struct ActionRequest<'a> {
        pub action: &'a str,
        pub item_id: &'a str,
    }

    impl<'a> ActionRequest<'a> {
        pub fn new(action: &'a str, item_id: &'a str) -> Self {
            ActionRequest { action: action, item_id: item_id }
        }
    }

    #[allow(unused_variables)] // for status codes
    pub fn send(config: &Config, actions: &[ActionRequest]) -> Result<String> {
        let json = do_send(config, actions).chain_err(|| ErrorKind::HttpActionCallFailed)?;

        Ok(json)
    }

    fn do_send(config: &Config, actions: &[ActionRequest]) -> Result<String> {
        let actions_json = serde_json::to_string(&actions).chain_err(|| "JSON serialization failed")?;

        let parameters = &[
            ("actions", actions_json),
            ("access_token", config.pocket.access_token.as_ref().unwrap().to_string()),
            ("consumer_key", config.pocket.consumer_key.to_string())
        ];
        let parameters_enc = serde_urlencoded::to_string(&parameters).chain_err(|| "URL serialization failed")?;
        let url = format!("https://getpocket.com/v3/send?{}", parameters_enc);

        let ssl = NativeTlsClient::new().chain_err(|| "Failed to create TLS client")?;
        let connector = HttpsConnector::new(ssl);
        let client = Client::with_connector(connector);
        let mut response = client
            .get(&url)
            .send()
            .chain_err(|| "Failed to finish HTTP request")?;

        let mut buffer = Vec::new();
        response.read_to_end(&mut buffer).chain_err(|| "Failed to read HTTP response")?;
        let json = str::from_utf8(&buffer).chain_err(|| "Data copying failed.")?;

        Ok(json.to_string())
    }
}