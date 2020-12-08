use crate::config_manager::ConfigManager;

use crate::inputs_set::InputsSet;
use futures::stream::unfold;
use futures::stream::BoxStream;
use iced_futures::subscription::Recipe;

use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};

pub struct OngoingSave {
    config_manager: ConfigManager,
}

impl OngoingSave {
    pub fn new(config_manager: ConfigManager) -> Self {
        Self { config_manager }
    }
}

struct OngoingSaveProgress {
    config_manager: ConfigManager,
    kind: OngoingSaveProgressKind,
}

enum OngoingSaveProgressKind {
    SaveToConfigFile,
    GenerateInputsSet,
    EnsureFixedLoaded(
        (InputsSet, BTreeMap<String, BTreeMap<String, String>>),
        usize,
    ),
    SavePackageFile((InputsSet, BTreeMap<String, BTreeMap<String, String>>)),
    Finished,
    Final,
}

impl<H: Hasher, I> Recipe<H, I> for OngoingSave {
    type Output = Option<String>;

    fn hash(&self, state: &mut H) {
        std::any::TypeId::of::<Self>().hash(state);
        self.config_manager.hash(state);
    }

    fn stream(self: Box<Self>, _input: BoxStream<'static, I>) -> BoxStream<'static, Self::Output> {
        Box::pin(unfold(
            OngoingSaveProgress {
                config_manager: self.config_manager.clone(),
                kind: OngoingSaveProgressKind::SaveToConfigFile,
            },
            |mut state| async move {
                match state.kind {
                    OngoingSaveProgressKind::SaveToConfigFile => {
                        state.config_manager.save_to_config_file().await;
                        state.kind = OngoingSaveProgressKind::GenerateInputsSet;
                        Some((Some("configuration saved".to_string()), state))
                    }
                    OngoingSaveProgressKind::GenerateInputsSet => {
                        let inputs_set =
                            state.config_manager.generate_inputs_set_for_enabled().await;
                        state.kind = OngoingSaveProgressKind::EnsureFixedLoaded(inputs_set, 0);
                        Some((Some("inputs set generated".to_string()), state))
                    }
                    OngoingSaveProgressKind::EnsureFixedLoaded(
                        (inputs_set, link_to_name),
                        position,
                    ) => {
                        if inputs_set.dependancies.len() <= position {
                            state.kind = OngoingSaveProgressKind::SavePackageFile((
                                inputs_set,
                                link_to_name,
                            ));
                            return Some((Some("finished loading fixed input".to_string()), state));
                        };
                        state
                            .config_manager
                            .ensure_fixed_is_loaded(&inputs_set.dependancies[position].distant)
                            .await;
                        let status = format!(
                            "finished to load fixed input from {:?}",
                            &inputs_set.dependancies[position].distant
                        );

                        state.kind = OngoingSaveProgressKind::EnsureFixedLoaded(
                            (inputs_set, link_to_name),
                            position + 1,
                        );
                        Some((Some(status), state))
                    }
                    OngoingSaveProgressKind::SavePackageFile((inputs_set, link_to_name)) => {
                        state
                            .config_manager
                            .write_nix_package_file(&inputs_set, &link_to_name)
                            .await;
                        state.kind = OngoingSaveProgressKind::Finished;
                        Some((Some("wrote nix package file".to_string()), state))
                    }
                    OngoingSaveProgressKind::Finished => {
                        state.kind = OngoingSaveProgressKind::Final;
                        Some((None, state))
                    }
                    OngoingSaveProgressKind::Final => None,
                }
            },
        ))
    }
}
