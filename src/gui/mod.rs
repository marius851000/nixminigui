mod application;
pub use application::{Flags, NixMiniGuiApp};

mod displayed_configuration;
pub use displayed_configuration::DisplayedConfiguration;

use crate::config_manager::ConfigManager;
pub struct AppSetting {
    pub config_manager: ConfigManager,
}

use crate::ongoing_save::OngoingSaveProgressMessage;
#[derive(Debug, Clone)]
pub enum Message {
    SwitchScreenInstallNew,
    SwitchScreenManageConfig,
    SelectedPotentialInstallTarget(String),
    EnableConfig(String),
    DisableConfig(String),
    ConfigurePackage(String),
    SetConfiguration(String, String, String), //config key, id, value
    ValidateChange,
    SetSaveProgress(Option<OngoingSaveProgressMessage>),
    Ignore,
    Todo,
}
