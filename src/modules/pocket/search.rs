use super::list;
use config::Config;
use utils::console::*;

use clap::{App, Arg, ArgMatches, SubCommand};

pub const NAME: &'static str = "search";

error_chain! {
    errors {
       PocketSearchFailed {
            description("failed to search Pocket articles")
            display("failed to search Pocket articles")
        }
    }
}


pub fn build_sub_cli() -> App<'static, 'static> {
    SubCommand::with_name(NAME)
        .about("Search in URL and title saved articles")
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
        .arg(Arg::with_name("sort")
            .long("sort")
            .takes_value(true)
            .possible_values(&["newest", "oldest", "title", "site"])
            .default_value("newest")
            .help("Select sort order"))
        .arg(Arg::with_name("since")
            .long("since")
            .takes_value(true)
            .help("Select articles modified since <duration> ago; e.g. '2w 3d 12m. Truncates fields from original JSON output."))
        .arg(Arg::with_name("until")
            .long("until")
            .takes_value(true)
            .help("Select articles added until <duration> ago; e.g. '2w 3d 12m. Truncates fields from original JSON output."))
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
            .required(true)
            .help("Search term"))
}

pub fn call(args: Option<&ArgMatches>, config: &Config) -> Result<()> {
    info(format!("Searching for articles ..."));
    list::call(args, config).chain_err( || ErrorKind::PocketSearchFailed)
}
