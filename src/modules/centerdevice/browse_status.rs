use config::Config;
use utils::console::*;

use clap::{App, Arg, ArgMatches, SubCommand};
use webbrowser;

pub const NAME: &'static str = "browse-status";

error_chain! {
    errors {
       CenterDeviceBrowseStatusFailed {
            description("failed to open CenterDevice status website")
            display("failed to open CenterDevice status website")
        }
    }
}


pub fn build_sub_cli() -> App<'static, 'static> {
    SubCommand::with_name(NAME)
        .about("Open CenterDevice Status website")
        .arg(Arg::with_name("details")
            .long("details")
            .help("Show details page"))
}

pub fn call(args: Option<&ArgMatches>, _: &Config) -> Result<()> {
    info(format!("Opening CenterDevice Status website ..."));
    let is_details = args.ok_or(false).unwrap().is_present("details");
    let result = match is_details {
        true  => webbrowser::open("http://status.centerdevice.de/details.html"),
        false => webbrowser::open("http://status.centerdevice.de")
    };
    result.map( |_| Ok(()) ).chain_err(|| ErrorKind::CenterDeviceBrowseStatusFailed)?
}

