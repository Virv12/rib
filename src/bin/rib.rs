use std::path::PathBuf;

use clap::Parser;
use serde::Deserialize;

use rib::Loc;

fn get_config_dir() -> PathBuf {
    dirs::config_dir().unwrap().join("rib.toml")
}

#[derive(Debug, Parser)]
struct Args {
    #[clap(short, long, default_value_os_t = get_config_dir())]
    config: PathBuf,
}

#[derive(Debug, Deserialize)]
struct Config {
    backup: Vec<Backup>,
}

#[derive(Debug, Deserialize)]
struct Backup {
    src: Loc,
    dst: Loc,
}

fn main() {
    env_logger::init();

    let args = Args::parse();

    let conf = std::fs::read(args.config).unwrap();
    let conf: Config = toml::from_str(std::str::from_utf8(&conf).unwrap()).unwrap();

    for backup in conf.backup {
        rib::backup(&backup.src, &backup.dst).unwrap();
    }
}
