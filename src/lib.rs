#![recursion_limit = "1024"]

extern crate clap;
extern crate curl;
#[macro_use] extern crate error_chain;
#[macro_use] extern crate itertools;
#[macro_use] extern crate serde_derive;
extern crate serde_json;
extern crate serde_urlencoded;
extern crate toml;
extern crate webbrowser;

pub mod config;
pub mod errors;
pub mod modules;
pub mod net;