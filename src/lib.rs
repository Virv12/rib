use std::{
    ffi::{OsStr, OsString},
    process::Command,
};

mod loc;

pub use loc::Loc;

type Result<T = (), E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

pub fn backup(src: &Loc, dst: &Loc) -> Result {
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

    dst.rename("current", &now_string)?;
    dst.link("last", &now_string)?;

    log::info!("backup {} complete", now_string);

    let backup_list = dst.get_list()?;
    let remove_list = remove_list(backup_list, now.timestamp());
    for backup in &remove_list {
        log::info!("removing backup {:?}", backup);
    }
    dst.remove_all(remove_list)?;

    Ok(())
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
