pub mod auth;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub consumer_key: String,
    pub access_token: Option<String>,
}

pub use self::auth::auth;