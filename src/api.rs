use std::path::Path;

use crate::{Context, ContextError, query::FnDef};

mod fn_def;
pub use fn_def::*;

#[derive(Debug, Clone, Default)]
pub struct ApiHeader {
    pub functions: Vec<ApiFnDef>,
}

impl ApiHeader {
    pub fn from_file(header_file: impl AsRef<Path>, template_file: impl AsRef<Path>) -> Result<Self, ContextError> {
        let mut s = Self::default();

        let ctx = Context::from_file(file)?;
        for m in FnDef::Match::from_node(ctx.root()) {
            s.functions.push(ApiFnDef::try_from(&m)?);
        }

        Ok(s)
    }

    pub fn from_buffer(header: impl Into<Vec<u8>>, template: impl Into<Vec<u8>>) -> Result<Self, ContextError> {

    }
}
