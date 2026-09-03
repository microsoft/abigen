use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbiConfig {
    pub exception: AbiExceptionConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbiExceptionConfig {
    pub value_type: String,
    pub nullref_value: String,
}
