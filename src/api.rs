use std::path::Path;

use crate::{Context, Error, query::FnDef};

mod fn_def;
pub use fn_def::*;

#[derive(Debug, Clone, Default)]
pub struct ApiHeader {
    pub functions: Vec<ApiFnDef>,
}

impl ApiHeader {
    pub fn from_file(header_file: impl AsRef<Path>) -> Result<Self, Error> {
        let header = std::fs::read(header_file)?;
        Self::from_buffer(header)
    }

    pub fn from_buffer(header: impl Into<Vec<u8>>) -> Result<Self, Error> {
        let mut s = Self::default();

        let ctx = Context::from_buffer(header)?;
        for m in FnDef::Match::from_node(ctx.root()) {
            s.functions.push(ApiFnDef::try_from(&m)?);
        }

        Ok(s)
    }
}
