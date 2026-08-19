use std::path::Path;

use tree_sitter::{QueryCursor, StreamingIterator};

use crate::{ApiFn, Context, ContextError, print_node, query};

#[derive(Debug, Clone, Default)]
pub struct ApiSurface {
    _fns: Vec<ApiFn>,
}

impl ApiSurface {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_header(&mut self, path: &Path) -> Result<(), ContextError> {
        let ctx = Context::try_from_file(path)?;
        let root = ctx.root();

        let mut qc = ctx.wrap(QueryCursor::new());
        let mut matches = qc.matches(&query::FN_DEF_QUERY, root);
        while let Some(qmatch) = matches.next() {
            print_node(qmatch.name, 6, 50);
        }

        Ok(())
    }
}

