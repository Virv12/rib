use std::{borrow::Cow, path::PathBuf};

use clap::Parser;
use log::LevelFilter;
use serde::Deserialize;

#[derive(Debug, Parser)]
enum Args {
    /// Backup from config file
    Config { config: PathBuf },
    /// Backup from command line
    Backup(Backup),
}

#[derive(Debug, Deserialize)]
struct Config {
    backup: Vec<Backup>,
}

#[derive(Debug, Clone, Deserialize, Parser)]
struct Backup {
    dst: String,

    #[clap(short, long)]
    src: Option<String>,

    #[serde(default)]
    #[clap(long)]
    one_file_system: bool,

    #[serde(default)]
    #[clap(long)]
    /// Additional arguments to pass to rsync
    extra_args: Vec<String>,
}

fn main() {
    env_logger::Builder::new()
        .filter_level(LevelFilter::Info)
        .parse_default_env()
        .init();

    let args = Args::parse();

    let backups = match &args {
        Args::Config { config } => {
            let conf = std::fs::read(config).unwrap();
            let conf: Config = toml::from_str(std::str::from_utf8(&conf).unwrap()).unwrap();
            Cow::Owned(conf.backup)
        }
        Args::Backup(backup) => Cow::Borrowed(std::slice::from_ref(backup)),
    };

    for backup in backups.as_ref() {
        let now = chrono::Utc::now();

        if let Some(src) = &backup.src {
            let res = rib::backup(
                &src.parse().unwrap(),
                &backup.dst.parse().unwrap(),
                backup.one_file_system,
                &backup.extra_args,
                now,
            );
            if let Err(e) = res {
                log::error!("Backup failed: {}", e);
                continue;
            }
        }

        let res = rib::cleanup(&backup.dst.parse().unwrap(), now);
        if let Err(e) = res {
            log::error!("Cleanup failed: {}", e);
        }
    }
}
