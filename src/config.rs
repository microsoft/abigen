use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::TypeName;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct AbiConfig {
    pub types: AbiTypeConfig,
    pub exception: AbiExceptionConfig,
}

impl AbiConfig {
    pub fn with_defaults(self) -> Self {
        Self::default().merged(self)
    }

    pub fn merged(mut self, other: Self) -> Self {
        self.merge(other);
        self
    }

    pub fn merge(&mut self, other: Self) {
        self.types.merge(other.types);
        self.exception.merge(other.exception);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AbiExceptionConfig {
    pub value_type: String,
    pub success_value: String,
    pub nullref_value: String,
    pub generic_failure_value: String,
    pub custom_values: HashMap<String, String>,
}

impl Default for AbiExceptionConfig {
    fn default() -> Self {
        Self {
            value_type: "int".to_string(),
            success_value: "0".to_string(),
            nullref_value: "2".to_string(),
            generic_failure_value: "1".to_string(),
            custom_values: HashMap::default(),
        }
    }
}

impl AbiExceptionConfig {
    pub fn merged(mut self, other: Self) -> Self {
        self.merge(other);
        self
    }

    pub fn merge(&mut self, other: Self) {
        self.value_type = other.value_type;
        self.success_value = other.success_value;
        self.nullref_value = other.nullref_value;
        self.generic_failure_value = other.generic_failure_value;
        self.custom_values.extend(other.custom_values);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AbiTypeConfig {
    pub builtin: HashSet<TypeName>,
    pub resolved: HashMap<TypeName, TypeName>,
    pub cast_mapped: HashMap<TypeName, TypeName>,
}

impl Default for AbiTypeConfig {
    fn default() -> Self {
        Self {
            builtin: ["int", "char", "float", "double"]
                .into_iter()
                .map(|t| t.into())
                .collect(),
            resolved: HashMap::new(),
            cast_mapped: [("bool", "int")]
                .into_iter()
                .map(|(x, y)| (x.into(), y.into()))
                .collect(),
        }
    }
}

impl AbiTypeConfig {
    pub fn with_defaults(self) -> Self {
        Self::default().merged(self)
    }

    pub fn merged(mut self, other: Self) -> Self {
        self.merge(other);
        self
    }

    pub fn merge(&mut self, other: Self) {
        self.builtin.extend(other.builtin);
        self.resolved.extend(other.resolved);
        self.cast_mapped.extend(other.cast_mapped);
    }
}
