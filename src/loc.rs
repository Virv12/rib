use std::{
    ffi::{OsStr, OsString},
    path::Path,
};

use serde::{de::Deserialize, Deserializer};

#[derive(Debug)]
pub enum Loc {
    Local {
        path: OsString,
    },
}

impl Loc {
    pub fn as_os_str(&self) -> &OsStr {
        match self {
            Loc::Local { path } => path,
        }
    }

    pub fn get_list(&self) -> Result<Vec<OsString>, std::io::Error> {
        match self {
            Loc::Local { path }=> {
                std::fs::read_dir(path)?
                    .map(|res| res.map(|e| e.file_name()))
                    .collect()
            }
        }
    }

    pub fn rename(&self, src: impl AsRef<OsStr>, dst: impl AsRef<OsStr>) -> Result<(), std::io::Error> {
        match self {
            Loc::Local { path }=> {
                std::fs::rename(Path::new(path).join(src.as_ref()), Path::new(path).join(dst.as_ref()))?;
            }
        }
        Ok(())
    }

    pub fn remove_all(&self, list: Vec<OsString>) -> Result<(), std::io::Error> {
        match self {
            Loc::Local { path }=> {
                for name in list {
                    std::fs::remove_dir_all(Path::new(path).join(name))?;
                }
            }
        }
        Ok(())
    }

    pub fn join(&self, oth: impl AsRef<OsStr>) -> Loc {
        match self {
            Loc::Local { path }=> {
                Loc::Local {
                    path: Path::new(path).join(oth.as_ref()).into(),
                }
            }
        }
    }
}

impl AsRef<OsStr> for Loc {
    fn as_ref(&self) -> &OsStr {
        self.as_os_str()
    }
}

impl<'de> Deserialize<'de> for Loc {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;
        Ok(Loc::Local { path: s.into() })
    }
}
