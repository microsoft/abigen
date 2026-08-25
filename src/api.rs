use std::path::Path;

use tree_sitter::QueryCursor;

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
        for qmatch in qc.matches(&query::FN_DEF_QUERY, root) {
            print_node(qmatch.name, 6, 50);
        }

        Ok(())
    }
}

