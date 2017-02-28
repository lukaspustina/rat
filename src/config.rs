use errors::*;
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
    pub fn from_file(file_path: &Path) -> Result<Config> {
        let mut config_file = File::open(file_path).chain_err(|| "Could not open config file.")?;
        let mut config_content = String::new();
        config_file.read_to_string(&mut config_content).chain_err(|| "Could not read config file.")?;

        let config: Config = toml::from_str(&config_content).chain_err(|| "Could not parse config file.")?;

        Ok(config)
    }
}