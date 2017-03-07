use config::{Config, OutputFormat};
use net::{curl_json, HttpVerb};
use utils::console::*;
use utils::output;

use clap::{App, Arg, ArgMatches, SubCommand};
use serde_json;
use std::str;

pub const NAME: &'static str = "status";

error_chain! {
    errors {
       CenterDeviceStatusFailed {
            description("failed to get CenterDevice status")
            display("failed to get CenterDevice status")
        }
    }
}

#[derive(Deserialize, Debug)]
enum Status {
    Okay,
    Warning,
    Failed,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
struct Rest {
    Status: Status,
    Timestamp: String,
    NotificationQueueClientPool: bool,
    FolderHealthCheckSensor: bool,
    DocumentQueueClientPool: bool,
    MetadataStoreResource: bool,
    NotificationStoreResource: bool,
    SearchEngineResource: bool,
    SecurityDataStoreResource: bool,
    SendEmailQueueClientPool: bool,
    UserdataStoreResource: bool,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
struct Auth {
    Status: Status,
    Timestamp: String,
    AuthServer: bool,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
struct WebClient {
    Status: Status,
    Timestamp: String,
    NotificationAlertingService: bool,
    RestServerSensor: bool,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
struct PublicLink {
    Status: Status,
    Timestamp: String,
    PublicLinkClient: bool,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
struct DistributorConsole {
    Status: Status,
    Timestamp: String,
    RestServerSensor: bool,
}

#[allow(non_camel_case_types)]
#[derive(Deserialize, Debug)]
enum PingDomStatus {
    up,
    down,
}

#[derive(Deserialize, Debug)]
struct Checks {
    checks: Vec<Vec<Check>>,
}


#[derive(Deserialize, Debug)]
struct Check {
    hostname: String,
    status: PingDomStatus,
    lasttesttime: String,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
struct PingDom {
    Status: Status,
    Timestamp: String,
    Checks: Checks,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
struct CenterDeviceStatus {
    Status: Status,
    Rest: Rest,
    Auth: Auth,
    WebClient: WebClient,
    PublicLink: PublicLink,
    DistributorConsole: DistributorConsole,
    PingDom: PingDom,
}

pub fn build_sub_cli() -> App<'static, 'static> {
    SubCommand::with_name(NAME)
        .about("Gets public centerdevice status from status server")
        .arg(Arg::with_name("details")
            .long("details")
            .help("Show detailed output"))
}

pub fn call(args: Option<&ArgMatches>, config: &Config) -> Result<()> {
    let details = args.ok_or(false).unwrap().is_present("details");
    info(format!("Getting CenterDevice Status ..."));
    status(config, details).chain_err(|| ErrorKind::CenterDeviceStatusFailed)
}

fn status(config: &Config, details: bool) -> Result<()> {
    let json = get_centerdevice_status_json()?;
    output(&json, &config.general.output_format, details)
}

fn get_centerdevice_status_json() -> Result<String> {
    let mut buffer = Vec::new();
    let url = "http://status.centerdevice.de/details.json";

    curl_json(url, HttpVerb::GET, None, None, Some(&mut buffer))
        .chain_err(|| ErrorKind::CenterDeviceStatusFailed)?;
    let json = str::from_utf8(&buffer).chain_err(|| "Data copying failed.")?;

    Ok(json.to_string())
}

fn output(json: &str, format: &OutputFormat, details: bool) -> Result<()> {
    match *format {
        OutputFormat::HUMAN => output_human(json, details),
        OutputFormat::JSON  => output::as_json(json)
            .chain_err(|| ErrorKind::CenterDeviceStatusFailed),
    }
}

fn output_human(json: &str, details: bool) -> Result<()> {
    let status: CenterDeviceStatus = serde_json::from_str(&json).chain_err(|| "JSON parsing failed")?;
    match (&status.Status, details) {
        (&Status::Okay, false) =>
            msgln(format!("CenterDevice status is {:?}.", status.Status)),
        (&Status::Okay, true) | (&Status::Warning, _) | (&Status::Failed, _) => {
            msgln(format!("CenterDevice status is {:?}.", status.Status));
            msgln(format!("+ Rest: {:?}", status.Rest.Status));
            msgln(format!("+ Auth: {:?}", status.Auth.Status));
            msgln(format!("+ WebClient: {:?}", status.WebClient.Status));
            msgln(format!("+ PublicLink: {:?}", status.PublicLink.Status));
            msgln(format!("+ DistributorConsole: {:?}", status.DistributorConsole.Status));
            msgln(format!("+ PingDom: {:?}", status.PingDom.Status));
        }
    }
    Ok(())
}

