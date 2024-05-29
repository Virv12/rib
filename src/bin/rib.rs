use std::path::PathBuf;

use serde::{Deserialize};

#[derive(Deserialize)]
struct Config {
    from: PathBuf,
    to: PathBuf,
}

fn main() {
    env_logger::init();

    let conf = std::fs::read("config.toml").unwrap();
    let conf: Config = toml::from_str(std::str::from_utf8(&conf).unwrap()).unwrap();

    rib::backup(&conf.from, &conf.to).unwrap();
}
