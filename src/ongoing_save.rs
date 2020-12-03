use crate::config_manager::ConfigManager;
use futures::stream::unfold;
use futures::stream::BoxStream;
use iced_futures::subscription::Recipe;
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
    Finished,
}

impl<H: Hasher, I> Recipe<H, I> for OngoingSave {
    type Output = String;

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
                        &state.config_manager.save_to_config_file().await;
                        state.kind = OngoingSaveProgressKind::Finished;
                        Some(("configuration saved".into(), state))
                    }
                    OngoingSaveProgressKind::Finished => {
                        println!("finished");
                        Some(("async".into(), state))
                    }
                }
            },
        ))
    }
}
