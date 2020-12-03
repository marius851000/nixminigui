use crate::config_manager::UserConfiguration;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, Hash)]
#[serde(tag = "type")]
pub enum Gate {
    Not { gate: Box<Gate> },
    And { gates: Vec<Gate> },
    Or { gates: Vec<Gate> },
    KeywordEqual { keyword: String, equal_to: String },
    KeywordTrue { keyword: String },
    KeywordFalse { keyword: String },
    True,
    False,
}

impl Default for Gate {
    fn default() -> Self {
        Gate::True
    }
}

impl Gate {
    pub fn evaluate(&self, configuration: &UserConfiguration) -> bool {
        //TODO: consider returning a result instead
        match self {
            Self::Not { gate } => !gate.evaluate(configuration),
            Self::And { gates } => {
                for gate in gates {
                    if !gate.evaluate(configuration) {
                        return false;
                    };
                }
                return true;
            }
            Self::Or { gates } => {
                for gate in gates {
                    if gate.evaluate(configuration) {
                        return true;
                    };
                }
                return false;
            }
            Self::KeywordEqual { keyword, equal_to } => {
                configuration.get(keyword).unwrap() == equal_to
            }
            Self::KeywordTrue { keyword } => configuration.get(keyword).unwrap() == "true",
            Self::KeywordFalse { keyword } => configuration.get(keyword).unwrap() != "true",
            Self::True => true,
            Self::False => false,
        }
    }
}
