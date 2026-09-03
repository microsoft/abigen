use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbiConfig {
    pub exception: AbiExceptionConfig, 
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbiExceptionConfig {
    pub result_type: String,
    pub throw_map: HashMap<String, String>,
}