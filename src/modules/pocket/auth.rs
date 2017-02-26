use super::Config;
use net::{curl, HttpVerb};
use serde_json;
use std::io;
use std::str;

static HEADERS: &'static [&'static str] = &["X-Accept: application/json", "Content-Type: application/json"];
static REDIRECT_URI: &'static str = "https://lukaspustina.github.io/rat/redirects/pocket.html";

#[derive(Serialize, Debug)]
struct Step1<'a> {
    consumer_key: &'a str,
    redirect_uri: &'a str,
}

#[derive(Deserialize, Debug)]
struct Step1Result {
    code: String,
}

#[derive(Serialize, Debug)]
struct Step3<'a> {
    consumer_key: &'a str,
    code: &'a str,
}

#[derive(Deserialize, Debug)]
struct Step3Result {
    access_token: String,
    username: String,
}

#[allow(unused_variables)] // for status codes
pub fn auth(config: &Config) {
    let consumer_key = &config.consumer_key;

    // Step 1 -- get code
    let mut buffer = Vec::new();
    let step_1 = Step1 { consumer_key: consumer_key, redirect_uri: REDIRECT_URI};
    // TODO: Only continue if 200
    let step_1_status_code = curl(
        "https://getpocket.com/v3/oauth/request",
        HttpVerb::POST,
        Some(&HEADERS),
        Some(&serde_json::to_string(&step_1).unwrap().into_bytes()),
        Some(&mut buffer)
    ).unwrap();
    let step_1_result: Step1Result = serde_json::from_slice(&buffer).unwrap();

    // Step 2 -- Wait for Web UI authentication
    println!(
        "Please authenticate at https://getpocket.com/auth/authorize?request_token={}&redirect_uri={} and then press return ...",
        step_1_result.code, REDIRECT_URI);
    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input);

    // Step 3 -- Exchange code for access token
    let mut buffer = Vec::new();
    let step_3 = Step3 { consumer_key: consumer_key, code: &step_1_result.code };
    // TODO: Only continue if 200
    let step_3_status_code = curl(
        "https://getpocket.com/v3/oauth/authorize",
        HttpVerb::POST,
        Some(&HEADERS),
        Some(&serde_json::to_string(&step_3).unwrap().into_bytes()),
        Some(&mut buffer)
    ).unwrap();
    let step_3_result: Step3Result = serde_json::from_slice(&buffer).unwrap();

    println!("Received access token '{}' for user '{}'.", step_3_result.access_token, step_3_result.username);
}