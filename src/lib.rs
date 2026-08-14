mod format;
pub use format::*;

mod context;
pub use context::*;

use std::sync::LazyLock;
use tree_sitter::Language;

pub static LANG: LazyLock<Language> = LazyLock::new(|| {
    tree_sitter_cpp::LANGUAGE.into()
});
