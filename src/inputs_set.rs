use crate::input::UpdatableInput;

use crate::nixtool::generate_dict_from_btreemap;
use std::collections::BTreeMap;

const SOURCE_PREFIX: &str = "source_";

#[derive(Hash, PartialEq, Eq, Clone)]
pub struct InputDeclaration {
    pub distant: UpdatableInput,
    pub depend_on: Vec<String>,
}

#[derive(PartialOrd, PartialEq, Eq, Ord)]
struct InputLoaded {
    distant: UpdatableInput,
    dependancies: BTreeMap<String, usize>,
}

#[derive(Default)]
pub struct InputsSet {
    dependancies: Vec<InputLoaded>,
}

//TODO: an id to name that can help do more human readible format (ensure they stay unique even if new data are added afterward)
impl InputsSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_group(
        &mut self,
        mut group: BTreeMap<String, InputDeclaration>,
    ) -> BTreeMap<String, String> {
        let mut loaded_dependancies: BTreeMap<String, usize> = BTreeMap::new();
        while !group.is_empty() {
            //TODO: rewrite with drain_filter when stabilized
            let mut to_remove: Vec<String> = Vec::new();
            for (k, v) in group.iter() {
                //do not try to add this if not all the dependancies are known
                for dependancy_key in &v.depend_on {
                    if !loaded_dependancies.contains_key(dependancy_key) {
                        continue;
                    };
                }
                loaded_dependancies.insert(k.to_string(), self.add_input(v, &loaded_dependancies));
                to_remove.push(k.to_string());
            }
            if to_remove.is_empty() {
                panic!("(TODO: clearer error message) infinite recursion detected in add_group of InputsList")
            };
            for remove in &to_remove {
                group.remove(remove).unwrap();
            }
        }
        loaded_dependancies
            .iter()
            .map(|(k, v)| (k, format!("{}{}", SOURCE_PREFIX, v)))
            .fold(BTreeMap::new(), |mut map, (k, v)| {
                map.insert(k.into(), v);
                map
            })
    }

    pub fn add_input(
        &mut self,
        input: &InputDeclaration,
        input_dependancies: &BTreeMap<String, usize>,
    ) -> usize {
        let mut dependancies = BTreeMap::new();
        for dep in &input.depend_on {
            dependancies.insert(dep.to_string(), *input_dependancies.get(dep).unwrap());
        }
        let loaded = InputLoaded {
            distant: input.distant.clone(),
            dependancies,
        };

        match self.dependancies.binary_search(&loaded) {
            Ok(position) => position,
            Err(new_position) => {
                self.dependancies.insert(new_position, loaded);
                new_position
            }
        }
    }

    pub async fn to_inputs_latest(&self) -> BTreeMap<String, String> {
        let mut result = BTreeMap::new();
        for (count, dep) in self.dependancies.iter().enumerate() {
            result.insert(
                format!("{}{}", SOURCE_PREFIX, count),
                format!(
                    "import {} {}",
                    dep.distant.get_latest().await.generate_nix_fetch(),
                    generate_dict_from_btreemap(&dep.dependancies.iter().fold(
                        BTreeMap::new(),
                        |mut map, (k, v)| {
                            map.insert(k.to_string(), format!("{}{}", SOURCE_PREFIX, v));
                            map
                        }
                    ))
                ),
            );
        }
        result
    }
}
