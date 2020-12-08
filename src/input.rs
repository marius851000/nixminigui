use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::nixtool::escape_string;

#[derive(Hash, PartialEq, Eq, Clone, PartialOrd, Ord, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum UpdatableInput {
    //Git(repo, ref)
    LocalPath {
        path: PathBuf,
        #[serde(default = "bool::default")]
        is_absolute: bool,
    },
    SystemWide {
        package: String,
    },
}

impl UpdatableInput {
    pub fn ensure_path_is_absolute(&mut self, base_dir: &Path) {
        match self {
            Self::LocalPath { path, .. } => {
                let mut path = path.clone();
                if path.is_relative() {
                    path = base_dir.join(&path);
                };
                *self = Self::LocalPath {
                    path,
                    is_absolute: true,
                };
            }
            _ => (),
        }
    }

    pub async fn get_latest(&self) -> FixedInput {
        match self {
            Self::LocalPath { path, is_absolute } => {
                if !is_absolute {
                    panic!("updateinput.get_latest, the path {:?} haven't been normalized (explicitly made absolute)", path);
                };
                FixedInput::LocalPath(path.to_string_lossy().to_string())
            }
            Self::SystemWide { package } => FixedInput::SystemWide(package.to_string()),
        }
    }
}

#[derive(Serialize, Deserialize, Hash, Clone, Debug)]
pub enum FixedInput {
    /// A path to a local folder or file. The file/folder itself is not fixed !
    LocalPath(String),
    /// A library in the nix search path. The library itself isn't fixed !
    SystemWide(String),
}

impl FixedInput {
    pub fn generate_nix_fetch(&self) -> String {
        match self {
            Self::LocalPath(absolute_path) => {
                format!("(builtins.toPath {})", escape_string(absolute_path))
            }
            Self::SystemWide(library) => format!("<{}>", library),
        }
    }
}
