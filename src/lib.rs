mod format;
pub use format::*;

mod context;
pub use context::*;

mod api;
pub use api::*;

mod config;
pub use config::*;

mod query;

#[cfg(test)]
pub(crate) mod test;

use std::sync::LazyLock;
use tree_sitter::Language;

pub(crate) static LANG: LazyLock<Language> = LazyLock::new(|| tree_sitter_cpp::LANGUAGE.into());
