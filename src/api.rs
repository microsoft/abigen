use std::{fmt::{self, Display}, path::Path};

use crate::{AbiConfig, Context, Error, query::FnDef};

mod fn_def;
pub use fn_def::*;

#[derive(Debug, Clone, Default)]
pub struct Api {
    pub functions: Vec<ApiFnDef>,
}

impl Api {
    pub fn from_file(header_file: impl AsRef<Path>) -> Result<Self, Error> {
        let header = std::fs::read(header_file)?;
        Self::from_buffer(header)
    }

    pub fn from_buffer(header: impl Into<Vec<u8>>) -> Result<Self, Error> {
        let mut s = Self::default();

        let ctx = Context::from_buffer(header)?;
        for m in FnDef::Match::from_node(ctx.root()) {
            s.functions.push(ApiFnDef::from(&m));
        }

        Ok(s)
    }

    pub fn to_c(self, config: &AbiConfig) -> Self {
        Self {
            functions: self.functions.into_iter().map(|f| f.to_c(config)).collect()
        }
    }

    pub fn header(&self) -> impl Display {
        fmt::from_fn(|f| {
            for func in &self.functions {
                writeln!(f, "{}", func.header())?;
            }
            Ok(())
        })
    }

    pub fn code(&self, config: &AbiConfig) -> impl Display {
        fmt::from_fn(|f| {
            for func in &self.functions {
                writeln!(f, "{}", func.code(config))?;
            }
            Ok(())
        })
    }
}
