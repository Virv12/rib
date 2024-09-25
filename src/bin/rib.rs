use std::path::PathBuf;

use clap::Parser;
use env_logger::Builder;
use log::LevelFilter;
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
    #[serde(default)]
    extra_args: Vec<String>,
    backup: Vec<Backup>,
}

#[derive(Debug, Deserialize)]
struct Backup {
    src: Loc,
    dst: Loc,
    #[serde(default)]
    one_file_system: bool,
}

fn main() {
    Builder::new()
        .filter_level(LevelFilter::Info)
        .parse_default_env()
        .init();

    let args = Args::parse();

    let conf = std::fs::read(args.config).unwrap();
    let conf: Config = toml::from_str(std::str::from_utf8(&conf).unwrap()).unwrap();

    for backup in conf.backup {
        rib::backup(
            &backup.src,
            &backup.dst,
            backup.one_file_system,
            &conf.extra_args,
        )
        .unwrap();
    }
}
