use std::{
    convert::Infallible,
    ffi::{OsStr, OsString},
    path::Path,
    process::Command,
    str::FromStr,
};

#[derive(Debug)]
pub enum Loc {
    Local { path: OsString },
    Ssh { path: OsString, colon: usize },
}

impl Loc {
    pub fn as_os_str(&self) -> &OsStr {
        match self {
            Loc::Local { path } => path,
            Loc::Ssh { path, .. } => path,
        }
    }

    pub fn rename(
        &self,
        src: impl AsRef<OsStr>,
        dst: impl AsRef<OsStr>,
    ) -> Result<(), std::io::Error> {
        match self {
            Loc::Local { path } => {
                std::fs::rename(
                    Path::new(path).join(src.as_ref()),
                    Path::new(path).join(dst.as_ref()),
                )?;
            }
            &Loc::Ssh { ref path, colon } => {
                let s = path.to_str().unwrap();
                let _ssh = Command::new("ssh")
                    .arg(s.get(..colon).unwrap())
                    .arg("mv")
                    .arg(Path::new(&s[colon + 1..]).join(src.as_ref()))
                    .arg(Path::new(&s[colon + 1..]).join(dst.as_ref()))
                    .status()?;
            }
        }
        Ok(())
    }

    pub fn link(
        &self,
        src: impl AsRef<OsStr>,
        dst: impl AsRef<OsStr>,
    ) -> Result<(), std::io::Error> {
        match self {
            Loc::Local { path } => {
                let _ln = Command::new("ln")
                    .arg("--force")
                    .arg("--no-dereference")
                    .arg("--symbolic")
                    .arg(dst)
                    .arg(Path::new(path).join(src.as_ref()))
                    .status()?;
            }
            &Loc::Ssh { ref path, colon } => {
                let s = path.to_str().unwrap();
                let _ssh = Command::new("ssh")
                    .arg(s.get(..colon).unwrap())
                    .arg("ln")
                    .arg("--force")
                    .arg("--no-dereference")
                    .arg("--symbolic")
                    .arg(dst)
                    .arg(Path::new(&s[colon + 1..]).join(src.as_ref()))
                    .status()?;
            }
        }
        Ok(())
    }

    pub fn get_list(&self) -> Result<Vec<OsString>, std::io::Error> {
        match self {
            Loc::Local { path } => std::fs::read_dir(path)?
                .map(|res| res.map(|e| e.file_name()))
                .collect(),
            &Loc::Ssh { ref path, colon } => {
                let s = path.to_str().unwrap();
                let ssh = Command::new("ssh")
                    .arg(s.get(..colon).unwrap())
                    .arg("ls")
                    .arg("-1")
                    .arg(&s[colon + 1..])
                    .output()?;
                let out = std::str::from_utf8(&ssh.stdout).unwrap();
                Ok(out.split('\n').map(|s| s.to_owned().into()).collect())
            }
        }
    }

    pub fn remove_all(&self, list: Vec<OsString>) -> Result<(), std::io::Error> {
        match self {
            Loc::Local { path } => {
                for name in list {
                    std::fs::remove_dir_all(Path::new(path).join(name))?;
                }
            }
            &Loc::Ssh { ref path, colon } => {
                let s = path.to_str().unwrap();
                let _ssh = Command::new("ssh")
                    .arg(s.get(..colon).unwrap())
                    .arg("rm")
                    .arg("-rf")
                    .args(list.into_iter().map(|t| Path::new(&s[colon + 1..]).join(t)))
                    .status()?;
            }
        }
        Ok(())
    }

    pub fn join(&self, oth: impl AsRef<OsStr>) -> Loc {
        match self {
            Loc::Local { path } => Loc::Local {
                path: Path::new(path).join(oth.as_ref()).into(),
            },
            &Loc::Ssh { ref path, colon } => Loc::Ssh {
                path: Path::new(path).join(oth.as_ref()).into(),
                colon,
            },
        }
    }
}

impl AsRef<OsStr> for Loc {
    fn as_ref(&self) -> &OsStr {
        self.as_os_str()
    }
}

impl FromStr for Loc {
    type Err = Infallible;

    fn from_str(path: &str) -> Result<Self, Self::Err> {
        let maybe_colon =
            path.find(|c: char| !c.is_ascii_alphanumeric() && c != '-' && c != '.' && c != '@');
        if let Some(colon) = maybe_colon {
            if path.as_bytes()[colon] == b':' {
                return Ok(Loc::Ssh {
                    path: path.into(),
                    colon,
                });
            }
        }
        Ok(Loc::Local { path: path.into() })
    }
}
