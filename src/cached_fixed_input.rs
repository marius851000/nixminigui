use crate::input::{FixedInput, UpdatableInput};
use std::collections::BTreeMap;

use async_std::fs::File;
use async_std::io::prelude::WriteExt;
use serde::{Deserialize, Serialize};

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

    pub async fn write_lock(&self, lock_file: &std::path::Path) {
        let mut file = File::create(lock_file).await.unwrap();
        let to_serialize: Vec<(UpdatableInput, FixedInput)> =
            self.cache.iter().fold(Vec::new(), |mut vec, (k, v)| {
                vec.push((k.clone(), v.clone()));
                vec
            });
        let to_write = serde_json::to_vec_pretty(&to_serialize).unwrap();
        file.write_all(&to_write).await.unwrap();
    }
}
