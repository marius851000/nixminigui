use crate::config_manager::UserConfiguration;

use crate::gate::Gate;
use serde::Deserialize;

use std::collections::BTreeMap;
use std::fs::File;

use std::io;

use std::path::Path;
use std::path::PathBuf;

use crate::input_distant::DistantInputs;
use crate::nixtool::{escape_string, generate_dict_from_btreemap};

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
#[derive(Clone, Hash)]
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

#[derive(Deserialize, Debug, Clone, Hash)]
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
    #[serde(default = "Effects::default")]
    pub effects: Effects,
}

#[derive(Deserialize, Debug, Clone, Hash)]
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

#[derive(Deserialize, Debug, Clone, Hash)]
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
    /// return the default value of this configuration kind, as a string. return none if it doesn't
    /// need to save anything.
    pub fn default_value(&self) -> Option<String> {
        match self {
            Self::Checkbox { default } => Some(if *default {
                "true".to_string()
            } else {
                "false".to_string()
            }),
            Self::RadioButton { default, .. } => Some(default.clone()),
            Self::Textbox { default, .. } => Some(default.clone()),
            Self::Group { .. } => None,
        }
    }
}

#[derive(Deserialize, Debug, Clone, Hash)]
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

#[derive(Default, Deserialize, Hash, Debug, Clone)]
pub struct Effects {
    #[serde(default = "BTreeMap::default")]
    pub inputs: BTreeMap<String, String>,
    pub package: Option<PackageEffect>,
}

#[derive(Deserialize, Hash, Debug, Clone)]
pub struct PackageEffect {
    path: String,
}

impl Effects {
    fn generate_input(
        &self,
        user_configuration: &UserConfiguration,
        base_directory: &Path,
    ) -> String {
        let mut inputs = self
            .inputs
            .iter()
            .map(|(k, v)| {
                (
                    escape_string(k),
                    DistantInputs::new(v.to_string(), base_directory).generate_nix_expression(),
                )
            })
            .fold(BTreeMap::new(), |mut map, (k, v)| {
                map.insert(k, v);
                map
            });
        inputs.insert(
            "\"params\"".to_string(),
            generate_dict_from_btreemap(&user_configuration.iter().fold(
                BTreeMap::new(),
                |mut map, (k, v)| {
                    map.insert(escape_string(k), escape_string(v));
                    map
                },
            )),
        );
        generate_dict_from_btreemap(&inputs)
    }

    pub fn generate_package(
        &self,
        user_configuration: &UserConfiguration,
        base_directory: &Path,
    ) -> Option<String> {
        if let Some(package) = &self.package {
            let inputs = self.generate_input(user_configuration, base_directory);
            let package_input = DistantInputs::new(package.path.clone(), base_directory);
            Some(format!(
                "( import {} {})",
                package_input.generate_nix_expression(),
                inputs
            ))
        } else {
            None
        }
    }
}
