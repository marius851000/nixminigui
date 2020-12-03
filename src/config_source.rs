use crate::gate::Gate;
use serde::Deserialize;

use std::fs::File;

use std::io;
use std::path::PathBuf;

const CONFIG_FILE_NAME: &str = "config.json";

quick_error! {
    #[derive(Debug)]
    pub enum LoadConfigError {
        CantReadFile { path: PathBuf, err :io::Error} {
            cause(err)
            display(me) -> ("can't read file {:?}: {}", path, err)
        }
        CantParseFile { path: PathBuf, err: serde_json::error::Error } {
            cause(err)
            display(me) -> ("can't parse file {:?}: {}", path, err)
        }
    }
}

/// Store the data from a configuration source. Configuration source is something that can either
/// be enabled or disabled, with optional additional option. The side effect are configured via a
/// nix expression.
pub struct ConfigSource {
    pub entry: ConfigEntry,
    pub folder_root: PathBuf,
}

impl ConfigSource {
    /// Create a new configuration source from a folder that contain it's data, including the
    /// root config.json
    pub fn new_from_path(folder_root: PathBuf) -> Result<Self, LoadConfigError> {
        Ok(ConfigSource {
            entry: ConfigEntry::new_from_path({
                let mut x = folder_root.clone();
                x.push(CONFIG_FILE_NAME);
                x
            })?,
            folder_root,
        })
    }
}

fn always_false() -> bool {
    false
}

#[derive(Deserialize, Debug)]
pub struct ConfigEntry {
    /// The config displayed name
    pub label: String,
    /// the id of this configuration, used if other configuration want to know if this
    /// configuration source is enabled.
    pub id: String,
    /// the description of the config
    pub desc: Option<String>,
    /// the group of people that manage this config file
    pub maintainers: Vec<String>,
    /// if this configuration source should always be enabled
    #[serde(default = "always_false")]
    pub always_enabled: bool,
    /// if this configuration source should be hidden (it can't be configured)
    #[serde(default = "always_false")]
    pub hidden: bool,
    /// the list of configuration of this configuration source
    #[serde(default = "Vec::new")]
    pub configurations: Vec<Configuration>,
}

#[derive(Deserialize, Debug)]
pub struct Configuration {
    /// the displayed name of this configuration
    pub label: String,
    /// the input name for the nix file of this configuration
    pub id: String,
    /// additional information to be displayed to the user
    pub info: Option<String>,
    /// the condition for this option to be displayed
    #[serde(default = "Gate::default")]
    pub condition: Gate,
    /// kind specific information
    pub kind: ConfigurationKind,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum ConfigurationKind {
    /// a checkbox
    Checkbox {
        /// the default value of this checkbox
        #[serde(default = "always_false")]
        default: bool,
    },
    /// a radio box list
    RadioButton {
        default: String,
        possibilities: Vec<RadioButtonPosibility>,
    },
    Textbox {
        #[serde(default = "String::new")]
        default: String,
    },
    Group {
        configurations: Vec<Configuration>,
    },
}

impl ConfigurationKind {
    pub fn default_value(&self) -> String {
        match self {
            Self::Checkbox { default } => {
                if *default {
                    "true".to_string()
                } else {
                    "false".to_string()
                }
            }
            Self::RadioButton { default, .. } => default.clone(),
            Self::Textbox { default, .. } => default.clone(),
            Self::Group { .. } => String::new(),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct RadioButtonPosibility {
    pub label: String,
    pub id: String,
}

impl ConfigEntry {
    pub fn new_from_path(config_path: PathBuf) -> Result<Self, LoadConfigError> {
        let configuration_file =
            File::open(&config_path).map_err(|err| LoadConfigError::CantReadFile {
                path: config_path.clone(),
                err,
            })?;
        let deserialized_entry = serde_json::from_reader(&configuration_file).map_err(|err| {
            LoadConfigError::CantParseFile {
                path: config_path,
                err,
            }
        })?;
        Ok(deserialized_entry)
    }
}
