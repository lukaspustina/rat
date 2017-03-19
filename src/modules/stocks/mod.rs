use config::{Config, OutputFormat};
use errors::*;
use utils::console::*;
use utils::output;

use clap::{App, Arg, ArgMatches, SubCommand};
use serde_json;
use std::fmt;

mod comdirect;

pub const NAME: &'static str = "stocks";


#[derive(Serialize, Debug)]
pub struct StockPrice {
    pub name: String,
    pub wkn: String,
    pub date: String,
    pub price: f32,
    pub currency: String,
}

pub fn build_sub_cli() -> App<'static, 'static> {
    SubCommand::with_name(NAME)
        .about("Stocks scrapes current stock prices from comdirect.de")
        .arg(Arg::with_name("search")
            .index(1)
            .required(true)
            .help("search term like company name, ISIN, WKN, or symbol"))
}

pub fn call(cli_args: Option<&ArgMatches>, config: &Config) -> Result<()> {
    let args = cli_args.unwrap();
    let search = args.value_of("search").unwrap().to_string();

    let stock_price = comdirect::scrape_stock_price(search)
        .chain_err(|| ErrorKind::ModuleFailed(NAME.to_string()))?;

    output(&stock_price, &config.general.output_format)
}

fn output(stock_price: &StockPrice, format: &OutputFormat) -> Result<()> {
    match *format {
        OutputFormat::HUMAN => output_human(stock_price),
        OutputFormat::JSON => {
            let json = serde_json::to_string(stock_price).chain_err(|| "Failed to serialize JSON")?;
            output::as_json(&json).chain_err(|| "Output failed")
        },
    }
}

impl fmt::Display for StockPrice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} ({}) is at {} {} on {}.", self.name, self.wkn, self.price, self.currency, self.date)
    }
}

fn output_human(stock_price: &StockPrice) -> Result<()> {
    msgln(format!("{}", stock_price));

    Ok(())
}
