use crate::config_manager::ConfigManager;
use crate::config_manager::UserConfiguration;
use crate::config_source::ConfigSource;

use crate::config_source::Configuration;
use crate::config_source::ConfigurationKind;

use crate::config_source::RadioButtonPosibility;
use iced::Checkbox;

use iced::Container;
use iced::Length;

use iced::text_input;
use iced::TextInput;
use iced::{
    button, executor, scrollable, Application, Button, Column, Command, Element, Row, Rule,
    Scrollable, Text,
};

pub struct NixMiniGuiApp {
    displayed_section: DisplayedSection,
    config_manager: ConfigManager,
}

pub struct AppSetting {
    pub config_manager: ConfigManager,
}

#[derive(Debug, Clone)]
pub enum Message {
    SwitchScreenInstallNew,
    SwitchScreenManageConfig,
    SelectedPotentialInstallTarget(String),
    EnableConfig(String),
    DisableConfig(String),
    ConfigurePackage(String),
    SetConfiguration(String, String, String), //config key, id, value
    Ignore,
    Todo,
}

pub type Flags = AppSetting;

impl Application for NixMiniGuiApp {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = Flags;

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            Self {
                displayed_section: DisplayedSection::new_select_config(&flags.config_manager),
                config_manager: flags.config_manager,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("nix mini gui")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::SwitchScreenInstallNew => {
                self.displayed_section =
                    DisplayedSection::new_selectable_config(&self.config_manager, None);
            }
            Message::SwitchScreenManageConfig => {
                self.displayed_section = DisplayedSection::new_select_config(&self.config_manager);
            }
            Message::SelectedPotentialInstallTarget(key) => {
                if let DisplayedSection::ChooseNewConfig {
                    selected_info,
                    selected_package,
                    ..
                } = &mut self.displayed_section
                {
                    *selected_info = Some(DisplayedConfigInfo::new(
                        &self.config_manager.get_config(&key).unwrap().0,
                    ));
                    *selected_package = Some(key);
                } else {
                    //TODO: use log (error)
                    println!("message SelectedPotentialInstallTarget received, but the screen isn't a ChooseNewConfig. Ignoring this message.");
                }
            }
            Message::EnableConfig(key) => {
                self.config_manager.enable_config(&key);
                self.displayed_section = DisplayedSection::new_select_config(&self.config_manager);
            }
            Message::ConfigurePackage(key) => {
                if let DisplayedSection::SelectConfig { selected, .. } = &mut self.displayed_section
                {
                    let config = &self.config_manager.get_config(&key).unwrap();
                    let displayed_configuration =
                        DisplayedConfiguration::new_from_source(&config.0, &config.2, key.clone());
                    *selected = Some(SelectConfigSelected {
                        key,
                        displayed_config_info: DisplayedConfigInfo::new(&config.0),
                        displayed_configuration,
                        scrollable_state: scrollable::State::new(),
                    });
                } else {
                    //TODO: use log(error)
                    println!("message ConfigurePackage received, but the screen isn't a SelectConfig. Ignoring this message.");
                }
            }
            Message::DisableConfig(key) => {
                self.config_manager.disable_config(&key);
                self.displayed_section = DisplayedSection::new_select_config(&self.config_manager);
            }
            Message::SetConfiguration(key, id, value) => {
                self.config_manager
                    .set_configuration(key.clone(), id, value);
                if let DisplayedSection::SelectConfig { selected, .. } = &mut self.displayed_section
                {
                    if let Some(selected) = selected {
                        if selected.key == key {
                            let config = &self.config_manager.get_config(&key).unwrap();
                            selected
                                .displayed_configuration
                                .update(&config.0, &config.2);
                        }
                    }
                }
            }
            Message::Ignore => (),
            Message::Todo => todo!(),
        }
        Command::none()
    }

    fn view(&mut self) -> Element<Self::Message> {
        self.displayed_section.view().into()
    }
}

pub struct SelectConfigSelected {
    key: String,
    displayed_config_info: DisplayedConfigInfo,
    displayed_configuration: DisplayedConfiguration,
    scrollable_state: scrollable::State,
}

pub enum DisplayedSection {
    SelectConfig {
        add_new_config_button_state: button::State,
        enabled_config: Vec<ButtonSelectableConfig>,
        selected: Option<SelectConfigSelected>,
        uninstall_button_state: button::State,
    },
    ChooseNewConfig {
        selectable_config: Vec<ButtonSelectableConfig>,
        selected_info: Option<DisplayedConfigInfo>,
        cancel_button_state: button::State,
        install_button_state: button::State,
        selected_package: Option<String>,
    },
}

impl DisplayedSection {
    fn new_selectable_config(
        config_manager: &ConfigManager,
        selected_package: Option<String>,
    ) -> Self {
        Self::ChooseNewConfig {
            selectable_config: config_manager
                .not_enabled_entry()
                .iter()
                .map(|info| {
                    ButtonSelectableConfig::new(
                        &info.0,
                        Message::SelectedPotentialInstallTarget(info.0.entry.id.to_string()),
                    )
                })
                .collect(),
            selected_info: None,
            cancel_button_state: button::State::new(),
            install_button_state: button::State::new(),
            selected_package,
        }
    }

    fn new_select_config(config_manager: &ConfigManager) -> Self {
        Self::SelectConfig {
            add_new_config_button_state: button::State::new(),
            enabled_config: config_manager
                .enabled_entry()
                .iter()
                .map(|info| {
                    ButtonSelectableConfig::new(
                        &info.0,
                        Message::ConfigurePackage(info.0.entry.id.to_string()),
                    )
                })
                .collect(),
            selected: None,
            uninstall_button_state: button::State::new(),
        }
    }

    fn view(&mut self) -> Element<Message> {
        match self {
            Self::SelectConfig {
                add_new_config_button_state,
                enabled_config,
                selected,
                uninstall_button_state,
            } => Row::new()
                .push(
                    Column::new()
                        .push::<Element<_>>({
                            let mut column = Column::new();
                            for config in enabled_config {
                                column = column.push(config.view());
                            }
                            column.into()
                        })
                        .push(
                            Button::new(
                                add_new_config_button_state,
                                Text::new("install new stuff"),
                            )
                            .on_press(Message::SwitchScreenInstallNew),
                        ),
                )
                .push(Rule::vertical(10))
                .push(
                    Column::new()
                        .push::<Element<_>>(if let Some(selected) = selected {
                            Column::new()
                                .push::<Element<_>>(
                                    Scrollable::new(&mut selected.scrollable_state)
                                        .push(selected.displayed_config_info.view())
                                        .push(selected.displayed_configuration.view())
                                        .height(Length::Fill)
                                        .into(),
                                )
                                .push::<Element<_>>(
                                    Container::new(
                                        Button::new(uninstall_button_state, Text::new("Uninstall"))
                                            .on_press(Message::DisableConfig(selected.key.clone())),
                                    )
                                    .into(), //TODO: red background style
                                )
                                .height(Length::Fill)
                                .into()
                        } else {
                            Container::new(Text::new("TODO: text for when no package are selected"))
                                .height(Length::Fill)
                                .into()
                        })
                        .push::<Element<_>>(Text::new("TODO: button validate change").into()),
                )
                .into(),
            Self::ChooseNewConfig {
                selectable_config,
                selected_info,
                cancel_button_state,
                install_button_state,
                selected_package,
            } => Row::new()
                .push(
                    Column::new()
                        .push(Text::new("TODO: search bar"))
                        //list of possible package
                        .push::<Element<_>>({
                            let mut possible_package = Vec::new();
                            for config in selectable_config {
                                possible_package.push(config.view());
                            }
                            Column::with_children(possible_package).into()
                        })
                        .push::<Element<_>>({
                            let mut button =
                                Button::new(install_button_state, Text::new("Install"));
                            if let Some(key) = selected_package {
                                button = button.on_press(Message::EnableConfig(key.to_string()));
                            };
                            button.into()
                        }),
                )
                .push(Rule::vertical(10))
                .push(
                    Column::new()
                        .push::<Element<_>>(if let Some(info) = selected_info {
                            info.view().into()
                        } else {
                            Text::new("TODO: message for when no config are selected").into()
                        })
                        .push(
                            Button::new(cancel_button_state, Text::new("Cancel"))
                                .on_press(Message::SwitchScreenManageConfig),
                        ),
                )
                .into(),
        }
    }
}

pub struct ButtonSelectableConfig {
    _id: String,
    message: Message,
    label: String,
    button_state: button::State,
}

impl ButtonSelectableConfig {
    pub fn new(config_source: &ConfigSource, message: Message) -> Self {
        Self {
            _id: config_source.entry.id.to_string(),
            message,
            label: config_source.entry.label.clone(),
            button_state: button::State::new(),
        }
    }

    fn view(&mut self) -> Element<Message> {
        Button::new(&mut self.button_state, Text::new(&self.label))
            .on_press(self.message.clone())
            .into()
    }
}

#[derive(Default)]
pub struct DisplayedConfigInfo {
    _id: String,
    label: String,
    desc: Option<String>,
    maintainers: Vec<String>,
}

impl DisplayedConfigInfo {
    pub fn new(config_source: &ConfigSource) -> Self {
        Self {
            _id: config_source.entry.id.clone(),
            label: config_source.entry.label.clone(),
            desc: config_source.entry.desc.clone(),
            maintainers: config_source.entry.maintainers.clone(),
        }
    }

    fn view(&self) -> Element<Message> {
        let mut column = Column::new().push(Text::new(self.label.to_string())); //TODO: format

        if let Some(desc) = &self.desc {
            column = column.push(Text::new(desc.to_string()));
        };

        if self.maintainers.len() != 0 {
            column = column.push({
                let mut row = Row::new().push(Text::new("maintainers :")).spacing(10);
                for maintainer in &self.maintainers {
                    row = row.push(Text::new(maintainer.to_string()));
                }
                row
            });
        };

        column.into()
    }
}

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
    fn new_from_source(
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

    fn update(&mut self, config_source: &ConfigSource, status: &UserConfiguration) {
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

    fn view(&mut self) -> Element<Message> {
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
