use std::{
    collections::HashSet,
    fmt::{self, Display},
    path::Path,
};

use crate::{AbiConfig, Context, Error, query::ClassDef};

mod fn_def;
pub use fn_def::*;

mod fn_arg;
pub use fn_arg::*;

mod typename;
pub use typename::*;

mod typ;
pub use typ::*;

mod class;
pub use class::*;

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
    pub classes: HashSet<ApiClassDef>,
}

impl Api {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_file(header_file: impl AsRef<Path>) -> Result<Self, Error> {
        let header = std::fs::read(header_file)?;
        Self::from_buffer(header)
    }

    pub fn from_buffer(header: impl Into<Vec<u8>>) -> Result<Self, Error> {
        let mut s = Self::default();

        let ctx = Context::from_buffer(header)?;
        for m in ClassDef::Match::from_node(ctx.root()) {
            s.classes.insert(ApiClassDef::from(&m));
        }

        Ok(s)
    }

    pub fn merged(mut self, other: Self) -> Self {
        self.merge(other);
        self
    }

    pub fn merge(&mut self, other: Self) {
        self.classes.extend(other.classes);
    }

    pub fn header(&self, config: &AbiConfig) -> impl Display {
        fmt::from_fn(|f| {
            let mut classes: Vec<_> = self.classes.iter().collect();
            classes.sort();

            writeln!(f, "{PRE}")?;

            for class in &classes {
                class.opaque_struct(f)?;
                writeln!(f)?;
            }
            writeln!(f)?;

            for class in &classes {
                class.header_methods(f, config)?;
                writeln!(f)?;
            }

            writeln!(f, "{POST}")
        })
    }

    pub fn code(&self, config: &AbiConfig) -> impl Display {
        fmt::from_fn(|f| {
            let needs_move = self.classes.iter().any(|c| {
                c.methods.iter().any(|m| {
                    m.args
                        .iter()
                        .any(|a| a.is_out == false && a.typ.relation == TypeRelation::Owned)
                })
            });

            let mut classes: Vec<_> = self.classes.iter().collect();
            classes.sort();

            if needs_move {
                writeln!(f, "#include <utility>")?;
            }

            writeln!(f, "{PRE}")?;

            for class in classes {
                class.code_methods(f, config)?;
                writeln!(f)?;
            }

            writeln!(f, "{POST}")
        })
    }
}
