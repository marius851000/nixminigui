use crate::distant_input::DistantInput;

use crate::nixtool::generate_dict_from_btreemap;
use std::collections::{BTreeMap, HashMap};

const SOURCE_PREFIX: &str = "source_";

#[derive(Hash, PartialEq, Eq, Clone)]
pub struct InputDeclaration {
    pub distant: DistantInput,
    pub depend_on: Vec<String>,
}

struct InputLoaded {
    distant: DistantInput,
    dependancies: BTreeMap<String, usize>,
}

#[derive(Default)]
pub struct InputsList {
    dependancies: Vec<InputLoaded>,
    dep_by_inputs: HashMap<InputDeclaration, usize>,
}

//TODO: an id to name that can help do more human readible format (ensure they stay unique even if new data are added afterward)
impl InputsList {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_group(
        &mut self,
        mut group: BTreeMap<String, InputDeclaration>,
    ) -> BTreeMap<String, String> {
        let mut loaded_dependancies: BTreeMap<String, usize> = BTreeMap::new();
        while group.len() != 0 {
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
            if to_remove.len() == 0 {
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
        if let Some(already_included_id) = self.dep_by_inputs.get(input) {
            return *already_included_id;
        } else {
            let new_id = self.dependancies.len();
            let mut dependancies = BTreeMap::new();
            for dep in &input.depend_on {
                dependancies.insert(dep.to_string(), *input_dependancies.get(dep).unwrap());
            }
            let loaded = InputLoaded {
                distant: input.distant.clone(),
                dependancies,
            };
            self.dependancies.push(loaded);
            self.dep_by_inputs.insert(input.clone(), new_id);
            new_id
        }
    }

    pub fn to_inputs(&self) -> BTreeMap<String, String> {
        let mut result = BTreeMap::new();
        for (count, dep) in self.dependancies.iter().enumerate() {
            result.insert(
                format!("{}{}", SOURCE_PREFIX, count),
                format!(
                    "import {} {}",
                    dep.distant.generate_nix_expression(),
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
