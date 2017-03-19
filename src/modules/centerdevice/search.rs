use super::client;

use config::{Config, OutputFormat};
use utils::console::*;
use utils::output;

use clap::{App, Arg, ArgMatches, SubCommand};
use serde_json;
use std::fmt;
use std::str;

pub const NAME: &'static str = "search";

error_chain! {
    errors {
        CenterDeviceSearchFailed {
            description("failed to search for documents")
            display("failed to search for documents")
        }

        OutputFailed {
            description("output failed")
            display("output failed")
        }
    }
}

pub fn build_sub_cli() -> App<'static, 'static> {
    SubCommand::with_name(NAME)
        .about("Search for documents in CenterDevice")
        .arg(Arg::with_name("filename")
            .long("filename")
            .short("f")
            .takes_value(true)
            .multiple(true)
            .help("Add filename to search"))
        .arg(Arg::with_name("tags")
            .long("tag")
            .short("t")
            .takes_value(true)
            .multiple(true)
            .help("Add tag to search"))
        .arg(Arg::with_name("fulltext")
            .index(1)
            .help("Add fulltext to search"))
}

pub fn call(args: Option<&ArgMatches>, config: &Config) -> Result<()> {
    let args = args.unwrap();

    let filenames: Option<Vec<&str>> = args.values_of("filename").map(|c| c.collect());
    let tags: Option<Vec<&str>> = args.values_of("tags").map(|c| c.collect());
    let fulltext = args.value_of("fulltext");

    info("Searching for documents ...");
    let json = client::search_documents(
        config.centerdevice.access_token.as_ref().unwrap(), filenames, tags, fulltext)
        .chain_err(|| ErrorKind::CenterDeviceSearchFailed)?;

    output(&json, &config.general.output_format)
}


fn output(json: &str, format: &OutputFormat) -> Result<()> {
    match *format {
        OutputFormat::HUMAN => output_human(json),
        OutputFormat::JSON => output::as_json(json).chain_err(|| ErrorKind::OutputFailed),
    }
}

#[derive(Deserialize, Debug)]
struct SearchResult {
    hits: u64,
    documents: Option<Vec<Documents>>,
}

#[derive(Deserialize, Debug)]
struct Documents {
    id: String,
    version: u32,
    filename: String,
    size: u64,
    #[serde(rename(deserialize = "upload-date"))] upload_date: String,
    #[serde(rename(deserialize = "version-date"))] version_date: String,
    representations: Representations,
}

#[derive(Deserialize, Debug)]
struct Representations {
    pdf: String,
    fulltext: String,
    jpg: String,
    png: String,
    mp4: String,
}

impl fmt::Display for Representations {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut reps = Vec::new();
        if self.pdf == "yes" { reps.push("pdf"); }
        if self.fulltext == "yes" { reps.push("fulltext"); }
        if self.jpg == "yes" { reps.push("jpg"); }
        if self.png == "yes" { reps.push("png"); }
        if self.mp4 == "yes" { reps.push("mp4"); }

        write!(f, "{:?}", reps)
    }
}

fn output_human(json: &str) -> Result<()> {
    let result: SearchResult = serde_json::from_str(json).chain_err(|| "JSON parsing failed")?;
    msgln(format!("Found {} document(s) matching the search parameters:", result.hits));

    if let Some(documents) = result.documents {
        for d in documents {
            msgln(format!("* {}: '{}', version {}, {} bytes, uploaded {}, created {}, {}",
                          d.id, d.filename, d.version, d.size, d.version_date, d.upload_date, d.representations));
        }
    }

    Ok(())
}