use std::path::Path;

use crate::{ApiFn, ContextError};

#[derive(Debug, Clone, Default)]
pub struct ApiSurface {
    _fns: Vec<ApiFn>,
}

impl ApiSurface {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_header(&mut self, _path: &Path) -> Result<(), ContextError> {
        todo!()
    }
}

