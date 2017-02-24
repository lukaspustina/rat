pub mod auth;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub consumer_key: String,
    pub redirect_uri: String,
}

pub use self::auth::auth;