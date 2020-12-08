use crate::input::{FixedInput, UpdatableInput};
use std::collections::BTreeMap;

#[derive(Hash, Default, Clone, Debug)]
pub struct CachedFixedInput {
    cache: BTreeMap<UpdatableInput, FixedInput>,
}

impl CachedFixedInput {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, k: UpdatableInput, v: FixedInput) -> Option<FixedInput> {
        self.cache.insert(k, v)
    }

    pub fn get(&self, k: &UpdatableInput) -> Option<&FixedInput> {
        self.cache.get(k)
    }

    pub fn remove(&mut self, k: &UpdatableInput) -> Option<FixedInput> {
        self.cache.remove(k)
    }

    pub async fn get_or_insert_latest(&mut self, k: &UpdatableInput) -> FixedInput {
        if let Some(get) = self.get(k) {
            get.clone()
        } else {
            let latest = k.get_latest().await;
            self.insert(k.clone(), latest.clone());
            self.get(k).unwrap().clone()
        }
    }
}
