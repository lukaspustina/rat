use errors::*;
use modules::{centerdevice, pocket, slack};

use std::fs::File;
use std::io::Read;
use std::path::Path;
use toml;

#[derive(Debug, Deserialize)]
#[serde(tag = "format")]
#[derive(PartialOrd, PartialEq, Eq)]
#[derive(Clone, Copy)]
pub enum OutputFormat {
    JSON,
    HUMAN,
}

impl<'a> From<&'a str> for OutputFormat {
    fn from(format: &'a str) -> Self {
        let format_sane: &str = &format.to_string().to_uppercase();
        match format_sane {
            "JSON" => OutputFormat::JSON,
            _ => OutputFormat::HUMAN
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "verbosity")]
#[derive(PartialOrd, PartialEq, Eq)]
#[derive(Clone, Copy)]
pub enum Verbosity {
    VERBOSE = 1,
    NORMAL = 2,
    QUIET = 3,
}

#[derive(Debug, Deserialize)]
pub struct GeneralConfig {
    pub cache_dir: String,
    pub output_format: OutputFormat,
    pub verbosity: Verbosity,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub general: GeneralConfig,
    pub centerdevice: centerdevice::CenterDeviceConfig,
    pub pocket: pocket::PocketConfig,
    pub slack: slack::SlackConfig,
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
