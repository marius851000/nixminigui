use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::nixtool::escape_string;

#[derive(Hash, PartialEq, Eq, Clone, PartialOrd, Ord)]
pub enum UpdatableInput {
    //Git(repo, ref)
    LocalPath(PathBuf),
    SystemWide(String),
}

impl UpdatableInput {
    pub fn new(input: String, base_dir: &Path) -> Self {
        if let Some(first_char) = input.chars().next() {
            if first_char == '.' || first_char == '/' || first_char == '~' {
                let mut path = PathBuf::from(input);
                if path.is_relative() {
                    path = base_dir.join(&path);
                };
                Self::LocalPath(path.canonicalize().unwrap())
            } else {
                //TODO: use something better (alias like in flake)
                Self::SystemWide(input)
            }
        } else {
            panic!("inputs can't be an empty String")
        }
    }

    pub async fn get_latest(&self) -> FixedInput {
        match self {
            Self::LocalPath(absolute_path) => {
                FixedInput::LocalPath(absolute_path.to_string_lossy().to_string())
            }
            Self::SystemWide(input) => FixedInput::SystemWide(input.to_string()),
        }
    }
}

#[derive(Serialize, Deserialize)]
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
