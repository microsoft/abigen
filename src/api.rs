use std::{
    collections::HashSet,
    fmt::{self, Display},
    path::Path,
};

use crate::{AbiConfig, Context, Error, format::fmt_wrap, query::ClassDef};

mod fn_def;
pub use fn_def::*;

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
                let name = &class.name;
                writeln!(
                    f,
                    "typedef struct {name} {name};",
                    name = fmt_wrap!(name.pascal_case),
                )?;
            }

            writeln!(f)?;

            for class in &classes {
                let mut methods: Vec<_> = class.methods.iter().collect();
                methods.sort();
                for method in methods {
                    method.header(f, config, class)?;
                    writeln!(f)?;
                }
                writeln!(f)?;
            }

            writeln!(f, "{POST}")
        })
    }

    pub fn code(&self, config: &AbiConfig) -> impl Display {
        fmt::from_fn(|f| {
            let mut classes: Vec<_> = self.classes.iter().collect();
            classes.sort();

            writeln!(f, "{PRE}")?;

            for class in &self.classes {
                let mut methods: Vec<_> = class.methods.iter().collect();
                methods.sort();

                for func in methods {
                    func.code(f, config, class)?;
                    writeln!(f)?;
                }
                writeln!(f)?;
            }

            writeln!(f, "{POST}")
        })
    }
}
