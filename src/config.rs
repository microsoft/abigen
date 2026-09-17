use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::TypeName;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AbiConfig {
    pub resolved_types: HashMap<TypeName, TypeName>,
    pub builtin_types: HashSet<TypeName>,
    pub exception: AbiExceptionConfig,
}

impl Default for AbiConfig {
    fn default() -> Self {
        Self {
            resolved_types: HashMap::new(),
            builtin_types: ["int", "char", "float", "double"].into_iter().map(|t| t.into()).collect(),
            exception: AbiExceptionConfig::default(),
        }
    }
}

impl AbiConfig {
    pub fn with_defaults(self) -> Self {
        let mut default = Self::default();
        default.resolved_types.extend(self.resolved_types);
        default.builtin_types.extend(self.builtin_types);
        default.exception = self.exception;
        default
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