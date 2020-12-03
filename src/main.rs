use iced::{Application, Settings};
use nixminigui::config_manager::ConfigManager;
use nixminigui::gui::{Flags, NixMiniGuiApp};
use std::path::PathBuf;

fn main() {
    let mut config_manager = ConfigManager::new();
    config_manager
        .add_configuration_source_from_path(PathBuf::from("./test_config/minetest"))
        .unwrap();
    config_manager
        .add_configuration_source_from_path(PathBuf::from("./test_config/factorio"))
        .unwrap();
    config_manager.enable_config("minetest");
    config_manager.enable_config("factorio");
    let flags = Flags { config_manager };

    NixMiniGuiApp::run(Settings::with_flags(flags)).unwrap();
}
