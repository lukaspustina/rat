use super::client;
use super::client::collections::CollectionsResult;

use cache::Cache;
use config::{Config, OutputFormat};
use utils::console::*;
use utils::output;

use clap::{App, Arg, ArgMatches, SubCommand};
use mime::Mime;
use mime_guess::guess_mime_type_opt;
use serde_json;
use std::path::Path;
use std::str;


pub const NAME: &'static str = "upload";

error_chain! {
    errors {
        CenterDeviceUploadFailed {
            description("failed to upload document")
            display("failed to upload document")
        }

        MimeTypeParsingFailed(mime_type: String) {
            description("failed to parse mime type")
            display("failed to parse mime type '{}'", mime_type)
        }

        MimeTypeGuessFailed {
            description("failed to guess mime type")
            display("failed to guess mime type")
        }

        OutputFailed {
            description("output failed")
            display("output failed")
        }
    }
}



pub fn build_sub_cli() -> App<'static, 'static> {
    SubCommand::with_name(NAME)
        .about("Uploads a document to CenterDevice")
        .arg(Arg::with_name("mime-type")
            .long("mime-type")
            .short("m")
            .takes_value(true)
            .help("Sets the mime type of document; will be guest if not specified"))
        .arg(Arg::with_name("filename")
            .long("filename")
            .short("f")
            .takes_value(true)
            .help("Sets filename of document different from original filename"))
        .arg(Arg::with_name("title")
            .long("title")
            .takes_value(true)
            .help("Sets title of document"))
        .arg(Arg::with_name("tags")
            .long("tag")
            .short("t")
            .takes_value(true)
            .multiple(true)
            .help("Sets tag for document"))
        .arg(Arg::with_name("collection")
            .long("collection")
            .short("c")
            .takes_value(true)
            .multiple(true)
            .help("Set collection id to add document to"))
        .arg(Arg::with_name("cached-collection-name")
            .long("Collection")
            .short("C")
            .takes_value(true)
            .multiple(true)
            .help("Set collection id by name from collection cache; cf. `centerdevice collections` help"))
        .arg(Arg::with_name("file")
            .index(1)
            .required(true)
            .help("file to upload"))
}

pub fn call(args: Option<&ArgMatches>, config: &Config) -> Result<()> {
    let args = args.unwrap();

    let file = args.value_of("file").unwrap();
    let file_path = Path::new(file);
    let mime_type: Mime = if let Some(mt) = args.value_of("mime-type") {
        mt.parse().map_err(|_| ErrorKind::MimeTypeParsingFailed(mt.to_string()))?
    } else {
        guess_mime_type_opt(&file_path).ok_or(ErrorKind::MimeTypeGuessFailed)?
    };
    let filename = if let Some(filename) = args.value_of("filename") {
        filename
    } else {
        file
    };
    let title = args.value_of("title");
    let tags: Option<Vec<&str>> = args.values_of("tags").map(|c| c.collect());
    let cache: CollectionsResult; // Needs to outlive collections
    let mut collections: Option<Vec<&str>> = args.values_of("collection").map(|c| c.collect());

    if args.is_present("cached-collection-name") {
        cache = Cache::new(config, super::NAME, super::collections::NAME)
            .load().chain_err(|| "Could not load collections cache")?;
        verboseln(format!("Loaded collections cache with {} items.", cache.collections.len()));

        let selected_col_names: Vec<&str> = args.values_of("cached-collection-name").map(|c| c.collect()).unwrap();
        for cn in selected_col_names {
            if let Some(c) = cache.collections.iter().find(|ref c| c.name == cn) {
                let mut cs: Vec<&str> = collections.unwrap_or_else(|| Vec::new());
                cs.push(&c.id);
                collections = Some(cs)
            } else {
                bail!("No collection with name '{}' found in collections cache.", cn)
            }
        }
    }

    info(format!("Uploading file '{}' ...", filename));
    let json = client::upload_document(
        config.centerdevice.access_token.as_ref().unwrap(),
        file_path,
        filename,
        mime_type,
        title,
        tags,
        collections
    ).chain_err(|| ErrorKind::CenterDeviceUploadFailed)?;

    output(&json, &config.general.output_format)
}


fn output(json: &str, format: &OutputFormat) -> Result<()> {
    match *format {
        OutputFormat::HUMAN => output_human(json),
        OutputFormat::JSON => output::as_json(json).chain_err(|| ErrorKind::OutputFailed),
    }
}

#[derive(Deserialize, Debug)]
struct UploadResult {
    id: String,
}

fn output_human(json: &str) -> Result<()> {
    let result: UploadResult = serde_json::from_str(json).chain_err(|| "JSON parsing failed")?;
    msgln(format!("Successfully uploaded document with id '{}'.", result.id));

    Ok(())
}
