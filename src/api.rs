use std::{
    collections::HashSet,
    fmt::{self, Display},
    path::Path,
};

use crate::{AbiConfig, Context, Error, query::FnDef};

mod fn_def;
pub use fn_def::*;

const PRE: &'static str = r#"
#ifdef __cplusplus
extern "C" {
#endif
"#;

const POST: &'static str = r#"
#ifdef __cplusplus
} // extern "C"
#endif
"#;

#[derive(Debug, Clone, Default)]
pub struct Api {
    pub functions: HashSet<ApiFnDef>,
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
            s.functions.insert(ApiFnDef::from(&m));
        }

        Ok(s)
    }

    pub fn header(&self, config: &AbiConfig) -> impl Display {
        fmt::from_fn(|f| {
            let mut fns: Vec<_> = self.functions.iter().collect();
            fns.sort();

            writeln!(f, "{PRE}")?;
            for func in fns {
                writeln!(f, "{}", func.header(config))?;
            }
            writeln!(f, "{POST}")
        })
    }

    pub fn code(&self, config: &AbiConfig) -> impl Display {
        fmt::from_fn(|f| {
            let mut fns: Vec<_> = self.functions.iter().collect();
            fns.sort();

            writeln!(f, "{PRE}")?;
            for func in fns {
                writeln!(f, "{}", func.code(config))?;
            }
            writeln!(f, "{POST}")
        })
    }
}
