#![recursion_limit = "1024"]

extern crate base64;
extern crate chrono;
extern crate clap;
extern crate crypto;
#[macro_use] extern crate error_chain;
extern crate humantime;
#[macro_use] extern crate hyper;
extern crate hyper_native_tls;
extern crate itertools;
#[macro_use] extern crate mime;
extern crate mime_guess;
extern crate mime_multipart;
extern crate select;
#[macro_use] extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate serde_urlencoded;
extern crate tabwriter;
extern crate term_painter;
extern crate toml;
extern crate webbrowser;

#[cfg(test)] #[macro_use] extern crate pretty_assertions;

pub mod cache;
pub mod config;
pub mod errors;
pub mod modules;
pub mod net;
pub mod utils;