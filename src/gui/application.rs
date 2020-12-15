use crate::config_manager::ConfigManager;
use crate::config_source::ConfigSource;
use crate::gui::DisplayedConfiguration;
use crate::ongoing_save::OngoingSaveProgressMessage;

use iced::Container;
use iced::Length;

use crate::gui::{AppSetting, Message};
use iced::Subscription;
use iced::{
    button, executor, scrollable, Application, Button, Column, Command, Element, Row, Rule,
    Scrollable, Text,
};

pub struct NixMiniGuiApp {
    displayed_section: DisplayedSection,
    config_manager: ConfigManager,
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
            Message::ValidateChange => {
                self.displayed_section =
                    DisplayedSection::new_progress_report("starting...".into());
            }
            Message::SetSaveProgress(Some(OngoingSaveProgressMessage::Done(progress_text))) => {
                self.displayed_section = DisplayedSection::new_progress_report(progress_text);
            }
            Message::SetSaveProgress(None) => {
                self.displayed_section = DisplayedSection::new_apply_finished();
            }
            Message::Ignore => (),
            Message::Todo => todo!(),
        }
        Command::none()
    }

    fn view(&mut self) -> Element<Self::Message> {
        self.displayed_section.view()
    }

    fn subscription(&self) -> Subscription<Message> {
        match &self.displayed_section {
            DisplayedSection::SaveProgressReport { .. } => {
                Subscription::from_recipe(self.config_manager.save_and_apply())
                    .map(Message::SetSaveProgress)
            }
            _ => Subscription::none(),
        }
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
        apply_change_state: button::State,
    },
    ChooseNewConfig {
        selectable_config: Vec<ButtonSelectableConfig>,
        selected_info: Option<DisplayedConfigInfo>,
        cancel_button_state: button::State,
        install_button_state: button::State,
        selected_package: Option<String>,
    },
    SaveProgressReport {
        progress_text: String,
    },
    ApplyFinished {
        continue_edit_state: button::State,
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
            apply_change_state: button::State::new(),
        }
    }

    fn new_progress_report(progress_text: String) -> Self {
        Self::SaveProgressReport { progress_text }
    }

    fn new_apply_finished() -> Self {
        Self::ApplyFinished {
            continue_edit_state: button::State::new(),
        }
    }

    fn view(&mut self) -> Element<Message> {
        match self {
            Self::SelectConfig {
                add_new_config_button_state,
                enabled_config,
                selected,
                uninstall_button_state,
                apply_change_state,
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
                        .push::<Element<_>>(
                            Button::new(apply_change_state, Text::new("apply changes"))
                                .on_press(Message::ValidateChange)
                                .into(),
                        ),
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
                            info.view()
                        } else {
                            Text::new("TODO: message for when no config are selected").into()
                        })
                        .push(
                            Button::new(cancel_button_state, Text::new("Cancel"))
                                .on_press(Message::SwitchScreenManageConfig),
                        ),
                )
                .into(),
            Self::SaveProgressReport { progress_text } => {
                Text::new(progress_text.to_string()).into()
            }
            Self::ApplyFinished {
                continue_edit_state,
            } => Column::new()
                .push::<Element<_>>(
                    Text::new("application finished successfully".to_string()).into(),
                )
                .push::<Element<_>>(
                    Button::new(continue_edit_state, Text::new("continue edit".to_string()))
                        .on_press(Message::SwitchScreenManageConfig)
                        .into(),
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

        if !self.maintainers.is_empty() {
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
