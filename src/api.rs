use std::path::Path;

use crate::{ContextError};

#[derive(Debug, Clone, Default)]
pub struct ApiSurface {
    _headers: Vec<ApiHeader>,
}

impl ApiSurface {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_header(&mut self, _path: &Path) -> Result<(), ContextError> {
        todo!()
    }
}

#[derive(Debug, Clone, Default)]
pub struct ApiHeader {

}