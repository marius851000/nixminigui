use crate::config_source::{ConfigSource, LoadConfigError};

use crate::distant_input::DistantInput;

use crate::inputs_list::InputDeclaration;
use crate::inputs_list::InputsList;

use crate::nixtool::escape_string;
use crate::nixtool::generate_dict_from_btreemap;
use crate::nixtool::to_nix_vec;
use crate::ongoing_save::OngoingSave;
use crate::saved_config::SavedConfig;
use std::collections::BTreeMap;
use std::path::PathBuf;

pub type UserConfiguration = BTreeMap<String, String>;

#[derive(Clone, Hash)]
pub struct ConfigManager {
    configs: Vec<(Option<ConfigSource>, bool, UserConfiguration)>, //source, enabled, additional configuration
    key_to_id: BTreeMap<String, usize>,
    user_config_path: PathBuf,
    package_nix_path: PathBuf,
}

impl ConfigManager {
    pub fn new(user_config_path: PathBuf, package_nix_path: PathBuf) -> Self {
        Self {
            configs: Vec::new(),
            key_to_id: BTreeMap::new(),
            user_config_path,
            package_nix_path,
        }
    }

    /// add a configuration source. Will replace the previous one if it has the same id.
    pub fn add_configuration_source(&mut self, config_source: ConfigSource) {
        let always_enabled = config_source.entry.always_enabled;
        let key = if let Some(key) = self.key_to_id.get(&config_source.entry.id) {
            self.configs[*key].0 = Some(config_source);
            self.configs[*key].1 = self.configs[*key].1 || always_enabled;
            *key
        } else {
            let position = self.configs.len();
            self.key_to_id
                .insert(config_source.entry.id.clone(), position);
            self.configs
                .push((Some(config_source), always_enabled, BTreeMap::new()));
            position
        };
        let entry = &mut self.configs[key];
        for configuration in &entry.0.as_ref().unwrap().entry.configurations {
            if !entry.2.contains_key(&configuration.id) {
                if let Some(value) = configuration.kind.default_value() {
                    entry.2.insert(configuration.id.clone(), value);
                }
            }
        }
    }

    /// load a configuration source from a path. Will replace the previous one if it has the same
    /// id.
    pub fn add_configuration_source_from_path(
        &mut self,
        root: PathBuf,
    ) -> Result<(), LoadConfigError> {
        let config = ConfigSource::new_from_path(root)?;
        self.add_configuration_source(config);
        Ok(())
    }

    pub fn get_config(&self, key: &str) -> Option<(&ConfigSource, bool, &UserConfiguration)> {
        if let Some(conf_id) = self.key_to_id.get(key) {
            if let Some(config_source) = &self.configs[*conf_id].0 {
                Some((
                    &config_source,
                    self.configs[*conf_id].1,
                    &self.configs[*conf_id].2,
                ))
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn not_enabled_entry(&self) -> Vec<(&ConfigSource, bool, &UserConfiguration)> {
        self.configs
            .iter()
            .filter(|x| !x.1)
            .filter_map(|x| {
                if let Some(config_source) = &x.0 {
                    Some((config_source, x.1, &x.2))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn enabled_entry(&self) -> Vec<(&ConfigSource, bool, &UserConfiguration)> {
        self.configs
            .iter()
            .filter(|x| x.1)
            .filter_map(|x| {
                if let Some(config_source) = &x.0 {
                    Some((config_source, x.1, &x.2))
                } else {
                    None
                }
            })
            .collect()
    }

    fn get_config_mut(
        &mut self,
        key: &str,
    ) -> &mut (Option<ConfigSource>, bool, UserConfiguration) {
        &mut self.configs[*self.key_to_id.get(key).unwrap()]
    }

    pub fn enable_config(&mut self, key: &str) {
        self.get_config_mut(key).1 = true;
    }

    pub fn disable_config(&mut self, key: &str) {
        let to_change = self.get_config_mut(key);
        if let Some(config_source) = &to_change.0 {
            if config_source.entry.always_enabled {
                return;
            } else {
                to_change.1 = false;
            }
        } else {
            to_change.1 = false;
        }
    }

    pub fn save_and_apply(&self) -> OngoingSave {
        OngoingSave::new(self.clone())
    }

    pub fn set_configuration(&mut self, key: String, id: String, value: String) {
        self.get_config_mut(&key).2.insert(id, value);
    }

    pub async fn save_to_config_file(&self) {
        let mut saved_config = SavedConfig::default();
        for (key, uid) in self.key_to_id.iter() {
            saved_config.configurations.insert(
                key.to_string(),
                (self.configs[*uid].1, self.configs[*uid].2.clone()),
            );
        }
        let save_content = serde_json::to_vec(&saved_config).unwrap();
        use async_std::fs::File;
        use async_std::prelude::*;
        let mut file = File::create(&self.user_config_path).await.unwrap();
        file.write_all(&save_content).await.unwrap();
    }

    pub async fn write_nix_package_file(&self) {
        let package_file = self.generate_nix_package_file();
        use async_std::fs::File;
        use async_std::prelude::*;
        let mut file = File::create(&self.package_nix_path).await.unwrap();
        file.write_all(package_file.as_bytes()).await.unwrap();
    }

    fn generate_nix_package_file(&self) -> String {
        let mut packages_string: Vec<String> = Vec::new();
        let mut inputs_list = InputsList::new();
        for dependancy in self.enabled_entry().iter() {
            if let Some(package) = &dependancy.0.entry.effects.package {
                let dependancies_declaration = dependancy
                    .0
                    .entry
                    .effects
                    .inputs
                    .iter()
                    .map(|(k, v)| {
                        (
                            k,
                            InputDeclaration {
                                distant: DistantInput::new(
                                    v.distant.to_string(),
                                    &dependancy.0.folder_root,
                                ),
                                depend_on: v.dependancies.clone(),
                            },
                        )
                    })
                    .fold(BTreeMap::new(), |mut map, (k, v)| {
                        map.insert(k.to_string(), v);
                        map
                    });
                let inputs = inputs_list.add_group(dependancies_declaration);
                let package_distant =
                    DistantInput::new(package.path.to_string(), &dependancy.0.folder_root);
                let mut package_inputs = inputs.iter().fold(BTreeMap::new(), |mut map, (k, v)| {
                    map.insert(escape_string(k), format!("inputs.{}", v));
                    map
                });
                package_inputs.insert(
                    "user_config".into(),
                    generate_dict_from_btreemap(&dependancy.2.iter().fold(
                        BTreeMap::new(),
                        |mut map, (k, v)| {
                            map.insert(escape_string(k), escape_string(v));
                            map
                        },
                    )),
                );
                let package_expression = format!(
                    "(import {} {})",
                    package_distant.generate_nix_expression(),
                    generate_dict_from_btreemap(&package_inputs)
                );
                packages_string.push(package_expression);
            }
        }
        format!(
            "{{}}:\nlet\ninputs = rec {};\nin\n{}",
            generate_dict_from_btreemap(&inputs_list.to_inputs()),
            to_nix_vec(&packages_string)
        )
    }

    pub fn load_config(&mut self) {
        let user_configs = SavedConfig::new_from_path(&self.user_config_path);
        for (key, (enabled, config)) in user_configs.configurations.iter() {
            if let Some(uid) = self.key_to_id.get(key) {
                self.configs[*uid].1 = *enabled;
                self.configs[*uid].2 = config.clone();
            } else {
                let uid = self.configs.len();
                self.key_to_id.insert(key.clone(), uid);
                self.configs.push((None, *enabled, config.clone()));
            }
        }
    }
}
