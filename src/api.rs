use std::path::Path;

use tree_sitter::{QueryCursor, StreamingIterator};

use crate::{ApiFn, Context, ContextError, InContext, print_node, query};

#[derive(Debug, Clone, Default)]
pub struct ApiSurface {
    fns: Vec<ApiFn>,
}

impl ApiSurface {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_header(&mut self, path: &Path) -> Result<(), ContextError> {
        let ctx = Context::try_from_file(path)?;
        let root = ctx.root();

        let mut qc = QueryCursor::new();
        let mut matches = root.matches(&query::FN_DEF_QUERY, &mut qc);
        while let Some(qmatch) = matches.item.next() {
            let node = InContext::new(qmatch.nodes_for_capture_index(0).next().unwrap(), &ctx);
            print_node(node, 6, 50);
        }

        Ok(())
    }
}

