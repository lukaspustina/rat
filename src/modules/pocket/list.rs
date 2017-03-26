use super::client;
use super::client::list::{HumanOutput, ListResult, Request};
use config::{Config, OutputFormat};
use utils::console::*;
use utils::output;
use utils::time;

use clap::{App, Arg, ArgMatches, SubCommand};
use serde_json;
use std::str;

pub const NAME: &'static str = "list";

error_chain! {
    errors {
       PocketListFailed {
            description("failed to list Pocket articles")
            display("failed to list Pocket articles")
        }
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
        .arg(Arg::with_name("search")
            .index(1)
            .help("Select articles with search term in title or url"))
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
        access_token: config.pocket.access_token.as_ref().unwrap(),
        state: state,
        tag: value,
        sort: sort,
        detailType: detail_type,
        search: search,
    };

    let json = client::list(config, &request, since, until).chain_err(|| ErrorKind::PocketListFailed)?;

    output(&json, &config.general.output_format, &human_output)
}


fn output(json: &str, format: &OutputFormat, human_output: &HumanOutput) -> Result<()> {
    match *format {
        OutputFormat::HUMAN => output_human(json, human_output),
        OutputFormat::JSON => output::as_json(json)
            .chain_err(|| ErrorKind::PocketListFailed),
    }
}

fn output_human(json: &str, human_output: &HumanOutput) -> Result<()> {
    let list: ListResult = serde_json::from_str(json).chain_err(|| "JSON parsing failed")?;

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
