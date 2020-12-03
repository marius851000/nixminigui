use crate::config_manager::UserConfiguration;
use crate::config_source::{ConfigSource, Configuration, ConfigurationKind, RadioButtonPosibility};
use crate::gui::Message;
use iced::{text_input, Checkbox, Column, Element, Rule, Text, TextInput};

pub enum DisplayedConfiguration {
    Group {
        configs: Vec<(bool, Self)>,
    },
    RadioButton {
        label: String,
        id: String,
        key: String,
        possibilities: Vec<RadioButtonPosibility>,
        selected: String,
    },
    Checkbox {
        label: String,
        id: String,
        key: String,
        enabled: bool,
    },
    Textbox {
        label: String,
        id: String,
        key: String,
        entered: String,
        placeholder: String,
        state: text_input::State,
    },
}

impl DisplayedConfiguration {
    pub fn new_from_source(
        config_source: &ConfigSource,
        status: &UserConfiguration,
        key: String,
    ) -> Self {
        Self::new_top_level_group(key, &config_source.entry.configurations, status)
    }

    fn new_top_level_group(
        conf_key: String,
        configs: &Vec<Configuration>,
        status: &UserConfiguration,
    ) -> Self {
        Self::Group {
            configs: configs
                .iter()
                .map(move |c| {
                    (
                        c.condition.evaluate(status),
                        Self::new_from_configuration(conf_key.clone(), c, status),
                    )
                })
                .collect(),
        }
    }

    fn new_from_configuration(
        conf_key: String,
        config: &Configuration,
        status: &UserConfiguration,
    ) -> Self {
        match &config.kind {
            ConfigurationKind::RadioButton {
                default,
                possibilities,
            } => Self::RadioButton {
                label: config.label.clone(),
                id: config.id.clone(),
                key: conf_key.clone(),
                possibilities: possibilities.clone(),
                selected: status
                    .get(&config.id)
                    .unwrap_or(&default.clone())
                    .to_string(),
            },
            ConfigurationKind::Checkbox { default } => Self::Checkbox {
                label: config.label.clone(),
                id: config.id.clone(),
                key: conf_key.clone(),
                enabled: status
                    .get(&config.id)
                    .map(|x| x == "true")
                    .unwrap_or(*default),
            },
            ConfigurationKind::Textbox { default } => Self::Textbox {
                label: config.label.clone(),
                id: config.id.clone(),
                key: conf_key.clone(),
                entered: status
                    .get(&config.id)
                    .map(|x| x.clone())
                    .unwrap_or(default.to_string()),
                state: text_input::State::new(),
                placeholder: default.to_string(),
            },
            ConfigurationKind::Group { configurations } => {
                let conf_key_clone = conf_key.clone();
                Self::Group {
                    configs: configurations
                        .iter()
                        .map(move |c| {
                            (
                                c.condition.evaluate(status),
                                Self::new_from_configuration(
                                    (&conf_key_clone).to_string(),
                                    c,
                                    status,
                                ),
                            )
                        })
                        .collect(),
                }
            }
        }
    }

    pub fn update(&mut self, config_source: &ConfigSource, status: &UserConfiguration) {
        if let Self::Group { configs } = self {
            for (disp, config) in configs
                .iter_mut()
                .zip(config_source.entry.configurations.iter())
            {
                disp.0 = config.condition.evaluate(status);
                disp.1.update_component(config, status);
            }
        } else {
            panic!("called update on a non Group element (that contains all the configurations)");
        }
    }

    fn update_component(&mut self, config: &Configuration, status: &UserConfiguration) {
        match self {
            Self::Group { configs } => {
                if let ConfigurationKind::Group { configurations } = &config.kind {
                    for (disp, sub_config) in configs.iter_mut().zip(configurations.iter()) {
                        disp.0 = config.condition.evaluate(status);
                        disp.1.update_component(sub_config, status);
                    }
                } else {
                    panic!()
                }
            }
            Self::RadioButton { selected, id, .. } => {
                if let Some(s) = status.get(id) {
                    *selected = s.clone();
                };
            }
            Self::Checkbox { enabled, id, .. } => {
                if let Some(e) = status.get(id) {
                    *enabled = if e == "true" { true } else { false };
                };
            }
            Self::Textbox { entered, id, .. } => {
                if let Some(e) = status.get(id) {
                    *entered = e.clone();
                }
            }
        }
    }

    pub fn view(&mut self) -> Element<Message> {
        match self {
            Self::Group { configs } => {
                let mut childrens = Vec::new();
                for (count, config) in configs
                    .iter_mut()
                    .filter(|x| x.0)
                    .map(|x| &mut x.1)
                    .enumerate()
                {
                    if count != 0 {
                        childrens.push(Rule::horizontal(10).into())
                    }
                    childrens.push(config.view().into())
                }
                Column::with_children(childrens).into()
            }
            Self::RadioButton {
                label,
                id,
                key,
                possibilities,
                selected,
            } => {
                let mut column =
                    Column::new().push::<Element<_>>(Text::new(format!("{} :", label)).into());
                for possibility in possibilities {
                    let checked = &possibility.id == selected;
                    column = column
                        .push::<Element<_>>(if checked {
                            Checkbox::new(checked, possibility.label.clone(), |_| Message::Ignore)
                                .into()
                        } else {
                            let key_clone = key.clone();
                            let id_clone = id.clone();
                            let value_clone = possibility.id.clone();
                            Checkbox::new(checked, possibility.label.clone(), move |_| {
                                Message::SetConfiguration(
                                    (&key_clone).to_string(),
                                    (&id_clone).to_string(),
                                    (&value_clone).to_string(),
                                )
                            })
                            .into()
                        })
                        .into();
                }
                column.into()
            }
            Self::Checkbox {
                label,
                id,
                key,
                enabled,
            } => {
                let key_clone = key.clone();
                let id_clone = id.clone();
                Checkbox::new(*enabled, label.to_string(), move |s| {
                    Message::SetConfiguration(
                        (&key_clone).to_string(),
                        (&id_clone).to_string(),
                        if s == true {
                            "true".to_string()
                        } else {
                            "false".to_string()
                        },
                    )
                })
                .into()
            }
            Self::Textbox {
                label,
                id,
                key,
                entered,
                placeholder,
                state,
            } => {
                let key_clone = key.clone();
                let id_clone = id.clone();
                Column::new()
                    .push(Text::new(label.to_string()))
                    .push(TextInput::new(state, &placeholder, &entered, move |v| {
                        Message::SetConfiguration(
                            (&key_clone).to_string(),
                            (&id_clone).to_string(),
                            v,
                        )
                    }))
                    .into()
            }
        }
    }
}
