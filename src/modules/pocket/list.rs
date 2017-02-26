use super::Config;
use net::{curl, HttpVerb};
use serde_json;
use std::str;

static HEADERS: &'static [&'static str] = &["Content-Type: application/json"];

#[derive(Serialize, Debug)]
struct Request<'a> {
    consumer_key: &'a str,
    access_token: &'a str,
}

#[allow(unused_variables)] // for status codes
pub fn list(config: &Config) {

    let mut buffer = Vec::new();
    let request = Request{ consumer_key: &config.consumer_key, access_token: &config.access_token.as_ref().unwrap() };
    // TODO: Only continue if 200
    let response_status_code = curl(
        "https://getpocket.com/v3/get",
        HttpVerb::POST,
        Some(&HEADERS),
        Some(&serde_json::to_string(&request).unwrap().into_bytes()),
        Some(&mut buffer)
    ).unwrap();

    let response_str = str::from_utf8(&buffer).unwrap();
    println!("{}", response_str);
}
