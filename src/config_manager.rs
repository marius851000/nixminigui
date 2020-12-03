use crate::config_source::{ConfigSource, LoadConfigError};
use std::collections::HashMap;
use std::path::PathBuf;

pub type UserConfiguration = HashMap<String, String>;

#[derive(Default)]
pub struct ConfigManager {
    configs: Vec<(ConfigSource, bool, UserConfiguration)>, //source, enabled, additional configuration (TODO:)
    key_to_id: HashMap<String, usize>,
}

impl ConfigManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// add a configuration source. Will replace the previous one if it has the same id.
    pub fn add_configuration_source(&mut self, config_source: ConfigSource) {
        let always_enabled = config_source.entry.always_enabled;
        let key = if let Some(key) = self.key_to_id.get(&config_source.entry.id) {
            self.configs[*key].0 = config_source;
            self.configs[*key].1 = self.configs[*key].1 || always_enabled;
            *key
        } else {
            let position = self.configs.len();
            self.key_to_id
                .insert(config_source.entry.id.clone(), position);
            self.configs
                .push((config_source, always_enabled, HashMap::new()));
            position
        };
        let entry = &mut self.configs[key];
        for configuration in &entry.0.entry.configurations {
            if !entry.2.contains_key(&configuration.id) {
                entry
                    .2
                    .insert(configuration.id.clone(), configuration.kind.default_value());
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

    pub fn get_config(&self, key: &str) -> Option<&(ConfigSource, bool, UserConfiguration)> {
        if let Some(conf_id) = self.key_to_id.get(key) {
            Some(&self.configs[*conf_id])
        } else {
            None
        }
    }

    pub fn not_enabled_entry(&self) -> Vec<&(ConfigSource, bool, UserConfiguration)> {
        self.configs.iter().filter(|x| !x.1).collect()
    }

    pub fn enabled_entry(&self) -> Vec<&(ConfigSource, bool, UserConfiguration)> {
        self.configs.iter().filter(|x| x.1).collect()
    }

    fn get_config_mut(&mut self, key: &str) -> &mut (ConfigSource, bool, UserConfiguration) {
        &mut self.configs[*self.key_to_id.get(key).unwrap()]
    }

    pub fn enable_config(&mut self, key: &str) {
        self.get_config_mut(key).1 = true;
    }

    pub fn disable_config(&mut self, key: &str) {
        let to_change = self.get_config_mut(key);
        if to_change.0.entry.always_enabled {
            return;
        } else {
            to_change.1 = false;
        }
    }

    pub fn set_configuration(&mut self, key: String, id: String, value: String) {
        self.get_config_mut(&key).2.insert(id, value);
    }
}
