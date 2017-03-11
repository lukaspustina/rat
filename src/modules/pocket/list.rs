use config::{Config, OutputFormat};
use net::{curl, HttpVerb};
use utils::console::*;
use utils::output;
use utils::time;

use chrono::{DateTime, NaiveDateTime, UTC};
use clap::{App, Arg, ArgMatches, SubCommand};
use serde_json;
use std::collections::HashMap;
use std::str;
use std::time::Duration;

pub const NAME: &'static str = "list";

static HEADERS: &'static [&'static str] = &["Content-Type: application/json"];

error_chain! {
    errors {
       PocketListFailed {
            description("failed to list Pocket articles")
            display("failed to list Pocket articles")
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Serialize, Debug)]
enum State {
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
enum Sort {
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
enum DetailType {
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
struct Request<'a> {
    consumer_key: &'a str,
    access_token: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")] state: Option<State>,
    #[serde(skip_serializing_if = "Option::is_none")] tag: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")] sort: Option<Sort>,
    detailType: DetailType,
    #[serde(skip_serializing_if = "Option::is_none")] search: Option<&'a str>,
}

#[derive(Debug)]
struct HumanOutput {
    id: bool,
    title: bool,
    url: bool,
    t_added: bool,
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


pub fn build_sub_cli() -> App<'static, 'static> {
    SubCommand::with_name(NAME)
        .about("List saved articles")
        .arg(Arg::with_name("details")
            .long("details")
            .short("d")
            .help("Select details for articles"))
        .arg(Arg::with_name("tag")
            .long("tag")
            .short("t")
            .takes_value(true)
            .help("Select articles tagged with <tag> to list"))
        .arg(Arg::with_name("state")
            .long("state")
            .short("s")
            .takes_value(true)
            .possible_values(&["unread", "archive", "all"])
            .default_value("unread")
            .help("Select articles to list"))
        .arg(Arg::with_name("since")
            .long("since")
            .takes_value(true)
            .help("Select articles added since <duration> ago; e.g. '2w 3d 12m. Truncates fields from original JSON output."))
        .arg(Arg::with_name("until")
            .long("until")
            .takes_value(true)
            .help("Select articles added until <duration> ago; e.g. '2w 3d 12m. Truncates fields from original JSON output."))
        .arg(Arg::with_name("sort")
            .long("sort")
            .takes_value(true)
            .possible_values(&["newest", "oldest", "title", "site"])
            .default_value("newest")
            .help("Select sort order"))
        .arg(Arg::with_name("output")
            .long("output")
            .short("o")
            .takes_value(true)
            .multiple(true)
            .require_delimiter(true)
            .possible_values(&["id", "title", "url", "t_added"])
            .default_value("id,title,url,t_added")
            .help("Select human output field; default all"))
}

pub fn call(args: Option<&ArgMatches>, config: &Config) -> Result<()> {
    let args = args.unwrap();

    let state = Some(args.value_of("state").unwrap().into());
    let value = if args.is_present("tag") {
        Some(args.value_of("tag").unwrap())
    } else {
        None
    };
    let sort = Some(args.value_of("sort").unwrap().into());
    let detail_type = args.is_present("details").into();
    let search = args.value_of("search");
    let since = if let Some(since) = args.value_of("since") {
        let unix_ts = time::parse_duration(since).chain_err(|| "Could not parse since duration")?;
        Some(unix_ts)
    } else {
        None
    };
    let until = if let Some(until) = args.value_of("until") {
        let unix_ts = time::parse_duration(until).chain_err(|| "Could not parse until duration")?;
        Some(unix_ts)
    } else {
        None
    };
    let human_output: HumanOutput = args.values_of("output").map(|c| c.collect::<Vec<&str>>()).unwrap().into();

    let request = Request {
        consumer_key: &config.pocket.consumer_key,
        access_token: &config.pocket.access_token.as_ref().unwrap(),
        state: state,
        tag: value,
        sort: sort,
        detailType: detail_type,
        search: search,
    };

    info(format!("Getting list of your articles ..."));
    let mut json = get(config, &request).chain_err(|| ErrorKind::PocketListFailed)?;

    if since.is_some() || until.is_some() {
        let list: ListResult = serde_json::from_str(&json).chain_err(|| "JSON parsing failed")?;
        info(format!("Filtering list of your {} article(s) ...", list.list.len()));
        let list = list.filter(&since, &until);
        json = serde_json::to_string(&list).chain_err(|| "JSON serialization failed")?;
    }

    output(&json, &config.general.output_format, &human_output)
}

#[allow(unused_variables)] // for status codes
fn get(config: &Config, request: &Request) -> Result<String> {
    let mut buffer = Vec::new();
    let request_json = serde_json::to_string(&request).chain_err(|| "JSON serialization failed")?;

    verboseln(format!("request = {}", request_json));
    // TODO: Only continue if 200
    let response_status_code = curl(
        "https://getpocket.com/v3/get",
        HttpVerb::POST,
        Some(&HEADERS),
        Some(&request_json.into_bytes()),
        Some(&mut buffer)
    ).chain_err(|| "Curl failed")?;
    let response_str = str::from_utf8(&buffer).chain_err(|| "Data copying failed.")?;

    Ok(response_str.to_string())
}

fn output(json: &str, format: &OutputFormat, human_output: &HumanOutput) -> Result<()> {
    match *format {
        OutputFormat::HUMAN => output_human(json, human_output),
        OutputFormat::JSON => output::as_json(json)
            .chain_err(|| ErrorKind::PocketListFailed),
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct ListResult {
    status: i32,
    complete: i32,
    list: HashMap<String, Article>,
}

impl ListResult {
    fn filter(self, since: &Option<Duration>, until: &Option<Duration>) -> Self {
        let mut new_list: HashMap<String, Article> = HashMap::new();
        for (k, v) in self.list {
            if let &Some(since) = since {
                if v.time_added().unwrap() < since { continue };
            }
            if let &Some(until) = until {
                if v.time_added().unwrap() > until { continue };
            }
            new_list.insert(k, v);
        }

        ListResult { status: self.status, complete: self.complete, list: new_list }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Article {
    item_id: String,
    resolved_title: String,
    resolved_url: String,
    time_added: String,
    time_updated: String,
}

impl Article {
    fn time_added(&self) -> Result<Duration> {
        let secs: u64 = self.time_added.parse().chain_err(|| "Failed to parse time")?;
        Ok(Duration::from_secs(secs))
    }

    fn human_display(&self, human_output: &HumanOutput) -> Result<String> {
        let mut outputs = Vec::new();
        if human_output.id {;
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

        let mut out_str: String = format!("{}", outputs.first().unwrap());
        outputs.remove(0);
        for o in outputs {
            out_str = out_str + &format!(", {}", o);
        }

        Ok(out_str)
    }
}

fn output_human(json: &str, human_output: &HumanOutput) -> Result<()> {
    let list: ListResult = serde_json::from_str(&json).chain_err(|| "JSON parsing failed")?;

    if list.status == 1 {
        msgln(format!("Received {} article(s).", list.list.values().len()));
    } else {
        msgln("Receiving articles failed.");
    }
    for a in list.list.values() {
        msgln(a.human_display(human_output).chain_err(|| "Human output failed")?);
    }

    Ok(())
}
