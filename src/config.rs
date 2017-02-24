use modules::{centerdevice, pocket};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use toml;

#[derive(Debug, Deserialize)]
#[serde(tag = "format")]
pub enum OutputFormat {
    JSON,
    HUMAN,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub centerdevice: centerdevice::Config,
    pub pocket: pocket::Config,
}

impl Config {
    pub fn from_file(file_path: &Path) -> Result<Config, Box<::std::error::Error>> {
        let mut config_file = File::open(file_path)?;
        let mut config_content = String::new();
        config_file.read_to_string(&mut config_content)?;

        let config: Config = toml::from_str(&config_content)?;

        Ok(config)
    }
}
