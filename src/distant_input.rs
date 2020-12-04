use std::path::{Path, PathBuf};

use crate::nixtool::escape_string;

#[derive(Hash, PartialEq, Eq, Clone)]
pub enum DistantInput {
    //Git(repo, branch, Option<rev>)
    LocalPath(PathBuf),
    SystemWide(String),
}

impl DistantInput {
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

    pub fn generate_nix_expression(&self) -> String {
        match self {
            Self::LocalPath(absolute_path) => format!(
                "(builtins.toPath {})",
                escape_string(absolute_path.to_str().unwrap())
            ),
            Self::SystemWide(input) => format!("<{}>", input),
        }
    }
}
