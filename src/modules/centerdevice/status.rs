use super::Config;
use config::OutputFormat;
use net::{curl_json, HttpVerb};

use clap::{App, ArgMatches, SubCommand};
use serde_json;
use std::str;

pub const NAME: &'static str = "status";

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
}

pub fn call(_: Option<&ArgMatches>, config: &Config) {
    status(config);
}

fn status(config: &Config) {
    let json = get_centerdevice_status_json();
    output(&json, &config.output_format);
}

fn get_centerdevice_status_json() -> String {
    let mut buffer = Vec::new();
    let url = "http://status.centerdevice.de/details.json";

    curl_json(url, HttpVerb::GET, None, None, Some(&mut buffer)).unwrap();
    let json = str::from_utf8(&buffer).unwrap();

    json.to_string()
}


fn output(json: &str, format: &OutputFormat) {
    match *format {
        OutputFormat::HUMAN => {
            let status: CenterDeviceStatus = serde_json::from_str(&json).unwrap();
            match status.Status {
                Status::Okay => println!("CenterDevice status is {:?}.", status.Status),
                Status::Warning|Status::Failed => {
                    println!("CenterDevice status is {:?}.", status.Status);
                    println!("+ Rest: {:?}", status.Rest.Status);
                    println!("+ Auth: {:?}", status.Auth.Status);
                    println!("+ WebClient: {:?}", status.WebClient.Status);
                    println!("+ PublicLink: {:?}", status.PublicLink.Status);
                    println!("+ DistributorConsole: {:?}", status.DistributorConsole.Status);
                    println!("+ PingDom: {:?}", status.PingDom.Status);
                }
            }
        }
        OutputFormat::JSON => {
            println!("{}", json)
        }
    }
}
