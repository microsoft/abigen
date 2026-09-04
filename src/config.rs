use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbiConfig {
    pub exception: AbiExceptionConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbiExceptionConfig {
    pub value_type: String,
    pub success_value: String,
    pub nullref_value: String,
    pub generic_value: String,
    pub custom_values: HashMap<String, String>,
}
