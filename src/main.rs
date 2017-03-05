extern crate rat;
extern crate clap;

use rat::config::*;
use rat::errors::*;
use rat::modules::centerdevice;
use rat::modules::pocket;
use rat::utils::*;

use clap::{Arg, ArgMatches, App, Shell};
use std::env;
use std::io;
use std::path::Path;

static BIN_NAME: &'static str = "rat";
static DEFAULT_CONFIG_FILE: &'static str = "rat.toml";
static VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() {
    if let Err(ref e) = run() {
        console::error(format!("{} failed because {}", BIN_NAME, e));

        for e in e.iter().skip(1) {
            console::error(format!("caused by: {}", e));
        }

        // The backtrace is not always generated. Try to run this example
        // with `RUST_BACKTRACE=1`.
        if let Some(backtrace) = e.backtrace() {
            console::error(format!("backtrace: {:?}", backtrace));
        }

        ::std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli_args = build_cli().get_matches();

    if cli_args.is_present("completions") {
        let shell= cli_args.value_of("completions").unwrap();
        build_cli().gen_completions_to(BIN_NAME, shell.parse::<Shell>().unwrap(), &mut io::stdout());
        return Ok(());
    }

    let default_config_path = format!("{}/.{}", env::home_dir().unwrap().display(), DEFAULT_CONFIG_FILE);
    let config_path = Path::new(cli_args.value_of("configfile").unwrap_or(&default_config_path));
    let mut config = Config::from_file(config_path)?;

    if cli_args.is_present("quiet") {
        config.general.verbosity = Verbosity::QUIET;
    } else if cli_args.is_present("verbose") {
        config.general.verbosity = Verbosity::VERBOSE;
    }
    console::init(config.general.verbosity);

    if let Some(output_format) = cli_args.value_of("output-format") {
        config.general.output_format = output_format.into();
    }

    if cli_args.is_present("show-config") {
        console::msg(format!("{:?}", &config))
    }

    let subcommand = cli_args.subcommand_name().ok_or(ErrorKind::NoCommandSpecified)?;
    call_module(subcommand, cli_args.subcommand_matches(subcommand), &config)
}


fn build_cli() -> App<'static, 'static> {
    let mut app = App::new("rat")
        .version(VERSION)
        .arg(Arg::with_name("configfile")
            .short("c")
            .long("config")
            .value_name("FILE")
            .help("Sets a custom config file")
            .takes_value(true))
        .arg(Arg::with_name("show-config")
            .long("show-config")
            .help("Prints config"))
        .arg(Arg::with_name("quiet")
            .short("q")
            .long("quiet")
            .conflicts_with("verbose")
            .help("Sets quiet mode"))
        .arg(Arg::with_name("verbose")
            .short("v")
            .long("verbose")
            .conflicts_with("quiet")
            .help("Set verbose mode"))
        .arg(Arg::with_name("output-format")
            .short("o")
            .long("output")
            .takes_value(true)
            .possible_values(&["human", "json"])
            .help("Sets output format"))
        .arg(Arg::with_name("completions")
            .long("completions")
            .takes_value(true)
            .hidden(true)
            .possible_values(&["bash", "fish", "zsh"])
            .help("The shell to generate the script for"));


    app = app.subcommand(centerdevice::build_sub_cli());
    app = app.subcommand(pocket::build_sub_cli());

    app
}

fn call_module(subcommand: &str, cli_args: Option<&ArgMatches>, config: &Config) -> Result<()> {
    match subcommand {
        centerdevice::NAME => centerdevice::call(cli_args, &config),
        pocket::NAME       => pocket::call(cli_args, &config),
        _ => Ok(())
    }
}
