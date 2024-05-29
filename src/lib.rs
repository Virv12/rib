use std::{
    ffi::{OsStr, OsString},
    path::Path,
    process::Command,
};

type Result<T = (), E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

pub fn backup(src: &Path, dst: &Path) -> Result {
    let now = chrono::offset::Utc::now();
    let now_string = now.to_rfc3339_opts(chrono::format::SecondsFormat::Secs, true);

    let _rsync = Command::new("rsync")
        .arg(src)
        .arg(dst.join("current"))
        .arg("--link-dest=../last")
        .arg("--archive")
        .arg("--compress")
        .arg("--delete-excluded")
        .arg("--progress")
        .arg("--verbose")
        .arg("--filter=dir-merge .backupignore")
        .arg("--filter=:- .gitignore")
        .status()?;

    std::fs::rename(dst.join("current"), dst.join(&now_string))?;

    let _ln = Command::new("ln")
        .arg("--force")
        .arg("--no-dereference")
        .arg("--symbolic")
        .arg(&now_string)
        .arg(dst.join("last"))
        .status()?;

    log::info!("backup {} complete", now_string);

    let backup_list = get_list(dst)?;
    let remove_list = remove_list(backup_list, now.timestamp());
    remove_all(dst, remove_list)?;

    Ok(())
}

fn get_list(dir: &Path) -> Result<Vec<OsString>, std::io::Error> {
    std::fs::read_dir(dir)?
        .map(|res| res.map(|e| e.file_name()))
        .collect()
}

fn remove_list(list: Vec<OsString>, mut now: i64) -> Vec<OsString> {
    fn try_conv(name: &OsStr) -> Option<i64> {
        let time: chrono::DateTime<chrono::Utc> = name.to_str()?.parse().ok()?;
        Some(time.timestamp())
    }

    let mut arr: Vec<_> = list
        .into_iter()
        .filter_map(|x| try_conv(&x).zip(Some(x)))
        .collect();
    arr.sort_unstable_by_key(|&(a, _)| a);

    while arr.last().map(|&(l, _)| l >= now) == Some(true) {
        arr.pop().unwrap();
    }

    let mut step = -1;
    let mut res = Vec::new();
    'outer: loop {
        for _ in 0..5 {
            while arr.last().map(|&(l, _)| l >= now) == Some(true) {
                let backup = arr.pop().unwrap();
                res.push(backup.1);
            }

            now = (now - 1) & step;

            if let Some(&(l, _)) = arr.last() {
                if l >= now {
                    arr.pop();
                }
            } else {
                break 'outer;
            }
        }
        step *= 2;
    }

    res
}

fn remove_all(dir: &Path, list: Vec<OsString>) -> Result<(), std::io::Error> {
    for name in list {
        log::info!("removing backup {}", name.to_str().unwrap());
        std::fs::remove_dir_all(dir.join(name))?;
    }
    Ok(())
}
