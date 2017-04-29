use super::client;

use config::{Config, OutputFormat};
use utils::console::*;
use utils::output;

use clap::{App, Arg, ArgMatches, SubCommand};
use serde_json;
use std::str;


pub const NAME: &'static str = "delete";

error_chain! {
    errors {
        CenterDeviceDeleteFailed {
            description("failed to delete document")
            display("failed to delete document")
        }

        OutputFailed {
            description("output failed")
            display("output failed")
        }
    }
}



pub fn build_sub_cli() -> App<'static, 'static> {
    SubCommand::with_name(NAME)
        .about("Deletes documents from CenterDevice")
        .arg(Arg::with_name("document-id")
            .index(1)
            .required(true)
            .multiple(true)
            .help("ID of document to delete"))
}

pub fn call(args: Option<&ArgMatches>, config: &Config) -> Result<()> {
    let args = args.unwrap();

    let document_ids: Vec<&str> = args.values_of("document-id").map(|c| c.collect()).unwrap();

    info(format!("Deleting documents with ids '{:?}' ...", document_ids));
    let json = client::delete_documents(
        config.centerdevice.access_token.as_ref().unwrap(),
        document_ids
    ).chain_err(|| ErrorKind::CenterDeviceDeleteFailed)?;

    output(&json, &config.general.output_format)
}


fn output(json: &str, format: &OutputFormat) -> Result<()> {
    match *format {
        OutputFormat::HUMAN => output_human(json),
        OutputFormat::JSON => output::as_json(json).chain_err(|| ErrorKind::OutputFailed),
    }
}

#[derive(Deserialize, Debug)]
struct DeleteResult {
    #[serde(rename(deserialize = "failed-documents"))] failed_documents: Vec<String>,
}

fn output_human(json: &str) -> Result<()> {
    let result: DeleteResult = serde_json::from_str(json).chain_err(|| "JSON parsing failed")?;
    msgln(format!("Failed to delete document with ids '{:?}'.", result.failed_documents));

    Ok(())
}
