use config::Config;
use utils::console::*;

use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use toml;

error_chain! {}

#[derive(Debug)]
pub struct Cache {
    cache_dir: String,
    filename: String,
}

impl Cache {
    pub fn new(config: &Config, module_name: &str, cache_name: &str) -> Self {
        let filename = format!("{}.{}.toml", module_name, cache_name);

        Cache { cache_dir: config.general.cache_dir.clone(), filename: filename }
    }

    pub fn load<T: Deserialize>(self) -> Result<T> {
        let mut path = PathBuf::from(&self.cache_dir);
        path.push(&self.filename);
        let mut file = File::open(path.into_os_string()).chain_err(|| "Could not open cache file.")?;
        let mut content = String::new();
        file.read_to_string(&mut content).chain_err(|| "Could not read config file.")?;

        let data: T = toml::from_str(&content).chain_err(|| "Could not parse cache file.")?;

        Ok(data)
    }

    pub fn write<T: Serialize>(&self, data: &T) -> Result<()> {
        self.check_or_create_cache_dir().chain_err(|| "Cache directory is unavailable")?;
        let toml = toml::to_string(&data).chain_err(|| "Failed to parse data to TOML")?;
        self.write_toml_file(&toml).chain_err(|| "Failed to write cache file")?;

        Ok(())
    }

    fn check_or_create_cache_dir(&self) -> Result<()> {
        let path = Path::new(&self.cache_dir);
        if !path.is_dir() {
            fs::create_dir_all(path).chain_err(|| "Could not create cache directory")?;
            verboseln(format!("Created cache directory '{}'", &self.cache_dir));
        }

        Ok(())
    }

    fn write_toml_file(&self, toml: &str) -> Result<()> {
        let mut path = PathBuf::from(&self.cache_dir);
        path.push(&self.filename);

        let mut file = File::create(path.into_os_string()).chain_err(|| "Failed to create cache file")?;
        file.write_all(&toml.as_bytes()).chain_err(|| "Failed to write data to cache file")?;

        Ok(())
    }
}

