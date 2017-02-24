use config::OutputFormat;
pub mod status;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub output_format: OutputFormat,
}

pub use self::status::status;