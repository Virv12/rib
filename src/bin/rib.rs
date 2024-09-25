use std::path::PathBuf;

use clap::Parser;
use env_logger::Builder;
use log::LevelFilter;
use serde::Deserialize;

#[derive(Debug, Parser)]
enum Args {
    //#[clap(short, long, default_value_os_t = get_config_dir())]
    //config: PathBuf,
    //#[command(subcommand)]
    //backup: Option<Backup>,
    Config(ConfigArgs),
    Backup(Backup),
}

#[derive(Debug, Parser)]
struct ConfigArgs {
    config: PathBuf,
}

#[derive(Debug, Deserialize)]
struct Config {
    backup: Vec<Backup>,
}

#[derive(Debug, Deserialize, Parser)]
struct Backup {
    src: String,
    dst: String,
    #[serde(default)]
    #[clap(long)]
    one_file_system: bool,
    #[serde(default)]
    #[clap(long)]
    extra_args: Vec<String>,
}

fn main() {
    Builder::new()
        .filter_level(LevelFilter::Info)
        .parse_default_env()
        .init();

    let args = Args::parse();

    let backups = match args {
        Args::Config(ConfigArgs { config }) => {
            let conf = std::fs::read(config).unwrap();
            let conf: Config = toml::from_str(std::str::from_utf8(&conf).unwrap()).unwrap();
            conf.backup
        }
        Args::Backup(backup) => vec![backup],
    };

    for backup in backups {
        let res = rib::backup(
            &backup.src.parse().unwrap(),
            &backup.dst.parse().unwrap(),
            backup.one_file_system,
            &backup.extra_args,
        );
        if let Err(e) = res {
            log::error!("Backup failed: {}", e);
        }
    }
}
