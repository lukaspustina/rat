use config::{Config, OutputFormat, Verbosity};
use errors::*;
use utils::console::*;
use utils::output;

use clap::{App, Arg, ArgMatches, SubCommand};
use indicatif::{ProgressBar, ProgressStyle};
use serde_json;
use std::io::Write;
use tabwriter::TabWriter;

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
        .arg(Arg::with_name("queries")
            .index(1)
            .required(true)
            .multiple(true)
            .help("search term like company name, ISIN, WKN, or symbol"))
}

pub fn call(cli_args: Option<&ArgMatches>, config: &Config) -> Result<()> {
    let args = cli_args.unwrap();
    let queries: Vec<&str> = args.values_of("queries").unwrap().map(|x| x).collect();

    let mut progress_bar: Option<ProgressBar> = None;
    let mut progress = None;
    if config.general.output_format == OutputFormat::HUMAN && config.general.verbosity <= Verbosity::NORMAL {
        let pb = ProgressBar::new(0);
        pb.set_style(ProgressStyle::default_bar()
            .template("[{elapsed_precise}] [{bar:40.blue/blue}] {pos}/{len} ({eta}) {msg} {spinner:.blue}")
        );
        pb.set_length(queries.len() as u64);
        progress_bar = Some(pb);
        progress = Some(|str: String| {
            progress_bar.as_ref().unwrap().set_message(&str);
        });
    };

    let mut stock_prices: Vec<StockPrice> = vec![];
    for query in queries {
        let r = comdirect::scrape_stock_price(query, progress.as_ref())
            .chain_err(|| format!("Failed to get stock price for query '{}'", query))?;
        if let Some(ref pb) = progress_bar {
            pb.inc(1);
        }
        stock_prices.push(r)
    }

    if let Some(ref pb) = progress_bar {
        pb.finish_with_message("done");
    };

    output(&stock_prices, &config.general.output_format)
}

fn output(stock_prices: &[StockPrice], format: &OutputFormat) -> Result<()> {
    match *format {
        OutputFormat::HUMAN => output_human(stock_prices),
        OutputFormat::JSON => {
            let json = serde_json::to_string(stock_prices).chain_err(|| "Failed to serialize JSON")?;
            output::as_json(&json).chain_err(|| "Output failed")
        },
    }
}

fn output_human(stock_prices: &[StockPrice]) -> Result<()> {
    let mut tw = TabWriter::new(vec![]);
    for ref sp in stock_prices {
        let _ = writeln!(
            &mut tw, "{}\t({})\tis at {:7.2} {}\ton {}", sp.name, sp.wkn, sp.price, sp.currency, sp.date);
    }
    tw.flush().unwrap();
    let out_str = String::from_utf8(tw.into_inner().unwrap()).unwrap();

    msgln(format!("{}", out_str));

    Ok(())
}
