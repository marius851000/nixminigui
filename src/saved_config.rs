use crate::config_manager::UserConfiguration;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

#[derive(Deserialize, Serialize, Default)]
pub struct SavedConfig {
    pub configurations: HashMap<String, (bool, UserConfiguration)>,
}

impl SavedConfig {
    pub fn new_from_path(path: &Path) -> Self {
        match File::open(path) {
            Ok(file) => serde_json::from_reader(file).unwrap(),
            Err(err) => {
                eprintln!("impossible to load configuration file: {:?}", err); //TODO: only ignore if the file doesn't exist, otherwise send the end user a message
                Self::default()
            }
        }
    }
}
