// https://mikedilger.github.io/formdata/formdata/index.html

use super::client;

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

    info(format!("Uploading file '{}' ...", filename));
    let json = client::upload_document(
        &config.centerdevice.access_token.as_ref().unwrap(), &file_path, &filename, mime_type, title, tags)
        .chain_err(|| ErrorKind::CenterDeviceUploadFailed)?;

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
    let result: UploadResult = serde_json::from_str(&json).chain_err(|| "JSON parsing failed")?;
    msgln(format!("Successfully uploaded document with id '{}'.", result.id));

    Ok(())
}