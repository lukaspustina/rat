use super::client;

use config::{Config, OutputFormat, Verbosity};
use utils::console::*;
use utils::output;

use clap::{App, Arg, ArgMatches, SubCommand};
use indicatif::{ProgressBar, ProgressStyle};
use std::str;

pub const NAME: &'static str = "download";

error_chain! {
    errors {
        CenterDeviceDownloadFailed {
            description("failed to download document")
            display("failed to download document")
        }

        OutputFailed {
            description("output failed")
            display("output failed")
        }
    }
}

pub fn build_sub_cli() -> App<'static, 'static> {
    SubCommand::with_name(NAME)
        .about("Download document from CenterDevice")
        .arg(Arg::with_name("filename")
            .long("filename")
            .short("f")
            .takes_value(true)
            .help("filename for download; default is original document filename"))
        .arg(Arg::with_name("id")
            .index(1)
            .required(true)
            .help("id of document to download"))
}

pub fn call(args: Option<&ArgMatches>, config: &Config) -> Result<()> {
    let args = args.unwrap();
    let filename: Option<&str> = args.value_of("filename");
    let doc_id = args.value_of("id").unwrap();

    info(format!("Downloading document {} ...", doc_id));

    let mut progress_bar: Option<ProgressBar> = None;
    let mut progress = None;
    if config.general.output_format == OutputFormat::HUMAN && config.general.verbosity <= Verbosity::NORMAL {
        let pb = ProgressBar::new(0);
        pb.set_style(ProgressStyle::default_bar()
            .template("[{elapsed_precise}] [{bar:40.blue/blue}] {bytes}/{total_bytes} ({eta}) {msg} {spinner:.blue}")
        );
        progress_bar = Some(pb);
        progress = Some(|total, delta| {
            let pb = progress_bar.as_ref().unwrap();
            pb.set_length(total as u64);
            pb.set_message("downloading");
            pb.inc(delta as u64);
        });
    };
    client::download_document(
        &config.centerdevice.api_base_url,
        config.centerdevice.access_token.as_ref().unwrap(), filename, doc_id, progress)
        .chain_err(|| ErrorKind::CenterDeviceDownloadFailed)?;

    if let Some(ref pb) = progress_bar {
        pb.finish_with_message("done");
    };

    output("{}", &config.general.output_format)
}

fn output(json: &str, format: &OutputFormat) -> Result<()> {
    match *format {
        OutputFormat::HUMAN => Ok(()),
        OutputFormat::JSON => output::as_json(json).chain_err(|| ErrorKind::OutputFailed),
    }
}
