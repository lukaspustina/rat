use super::client;
use super::client::collections::CollectionsResult;

use config::{Config, OutputFormat};
use utils::console::*;
use utils::output;

use clap::{App, Arg, ArgMatches, SubCommand};
use serde_json;
use std::io::Write;
use std::str;
use tabwriter::TabWriter;

pub const NAME: &'static str = "collections";

error_chain! {
    errors {
        CenterDeviceCollectionFailed {
            description("failed to search for collections")
            display("failed to search for collections")
        }

        OutputFailed {
            description("output failed")
            display("output failed")
        }
    }
}

pub fn build_sub_cli() -> App<'static, 'static> {
    SubCommand::with_name(NAME)
        .about("Search for collections in CenterDevice")
        .arg(Arg::with_name("name")
            .long("name")
            .short("n")
            .takes_value(true)
            .help("Search for collections with this name"))
        .arg(Arg::with_name("public_collections")
            .long("public-collections")
            .short("p")
            .help("Includes public collections in search"))
        .arg(Arg::with_name("filter")
            .index(1)
            .takes_value(true)
            .help("filters collection names by regex"))
}

pub fn call(args: Option<&ArgMatches>, config: &Config) -> Result<()> {
    let args = args.unwrap();

    let name: Option<&str> = args.value_of("name");
    let include_public = args.is_present("public_collections");
    let filter = args.value_of("filter");

    info("Searching for collections ...");
    if include_public {
        info("Including public collections");
    }
    let json = client::search_collections(
        config.centerdevice.access_token.as_ref().unwrap(), name, include_public, filter)
        .chain_err(|| ErrorKind::CenterDeviceCollectionFailed)?;

    output(&json, &config.general.output_format)
}

fn output(json: &str, format: &OutputFormat) -> Result<()> {
    match *format {
        OutputFormat::HUMAN => output_human(json),
        OutputFormat::JSON => output::as_json(json).chain_err(|| ErrorKind::OutputFailed),
    }
}

fn output_human(json: &str) -> Result<()> {
    let result: CollectionsResult = serde_json::from_str(json).chain_err(|| "JSON parsing failed")?;
    msgln(format!("Found {} collection(s) matching the search parameters:", result.collections.len()));

    let mut tw = TabWriter::new(vec![]);
    for c in result.collections {
        let visibility = if c.public {
            "public"
        } else {
            "private"
        };
        let _ = write!(&mut tw, "* {}:\t'{}'\t[{}]\n", c.id, c.name, visibility);
    }
    tw.flush().unwrap();//.chain_err("|| Failed to create output table");
    let written = String::from_utf8(tw.into_inner().unwrap()).unwrap();
    msgln(written);

    Ok(())
}