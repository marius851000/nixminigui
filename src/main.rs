use iced::{Application, Settings};
use nixminigui::config_manager::ConfigManager;
use nixminigui::gui::{Flags, NixMiniGuiApp};
use std::path::PathBuf;

fn main() {
    let mut config_manager = ConfigManager::new(
        PathBuf::from("./nixminigui.json"),
        PathBuf::from("./lockfile.json"),
        PathBuf::from("./packages.nix"),
    );
    config_manager.load_config();
    config_manager.load_lock();
    config_manager
        .add_configuration_source_from_path(PathBuf::from("./test_config/minetest"))
        .unwrap();
    config_manager
        .add_configuration_source_from_path(PathBuf::from("./test_config/factorio"))
        .unwrap();

    let flags = Flags { config_manager };

    NixMiniGuiApp::run(Settings::with_flags(flags)).unwrap();
}
