extern crate rat;

use rat::config::Config;
use rat::modules::centerdevice;
use rat::modules::pocket;
use std::path::Path;

fn main() {
    let config = Config::from_file(Path::new("rat.toml")).unwrap();

    centerdevice::status(&config.centerdevice);
    pocket::auth(&config.pocket);
}
