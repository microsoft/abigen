use std::path::Path;

use crate::{Context, ContextError, query::FnDef};

mod fn_def;
pub use fn_def::*;

#[derive(Debug, Clone, Default)]
pub struct ApiHeader {
    pub functions: Vec<ApiFnDef>,
}

impl ApiHeader {
    pub fn from_file(file: &Path) -> Result<Self, ContextError> {
        let mut s = Self::default();

        let ctx = Context::from_file(file)?;
        for m in FnDef::Match::from_node(ctx.root()) {
            s.functions.push(m.try_into()?);
        }

        Ok(s)
    }
}
