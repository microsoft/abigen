pub mod format;
mod context;

use std::sync::LazyLock;
use tree_sitter::{Language, Node, Parser, Query, QueryCursor, StreamingIterator, Tree, TreeCursor};
pub use context::*;

static LANG: LazyLock<Language> = LazyLock::new(|| {
    tree_sitter_cpp::LANGUAGE.into()
});
