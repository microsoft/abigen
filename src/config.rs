use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AbiConfig {
    pub exception: AbiExceptionConfig,
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
