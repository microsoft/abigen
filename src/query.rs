use std::{pin::Pin, sync::LazyLock};
use tree_sitter::{Node, Query, QueryCursor, QueryMatch, QueryMatches, StreamingIterator};

use crate::{Context, InContext};

static FN_DEF_QUERY: LazyLock<FnDefQuery> = LazyLock::new(|| {
    let query = Query::new(&crate::LANG, r#"
        ((_)
            type: (_) @ret_t
            declarator: (identifier) @name
        ) @decl
    "#).expect("Failed to compile fn_def");

    let return_t = query.capture_index_for_name("return_t")
        .expect("Failed to get capture return_t");

    let name = query.capture_index_for_name("name")
        .expect("Failed to get capture name");

    FnDefQuery {
        query,
        return_t,
        name,
    }
});


pub struct FnDefQuery {
    pub query: Query,
    pub return_t: u32,
    pub name: u32,
    // pub ns: u32,
    // pub arg_t: u32,
    // pub arg_name: u32,
}

impl FnDefQuery {
    pub fn matches<'q, 't>(&'q self, node: InContext<'t, Node<'t>>) -> FnDefQueryMatches<'q, 't> {
        let mut temp = QueryCursor::new();
        let mut ret = FnDefQueryMatches {
            query: self,
            cursor: Box::pin(QueryCursor::new()),
            matches: temp.matches(&self.query, node.item, node.context.text()),
        };
        ret.matches = ret.cursor.matches(&self.query, node.item, node.context.text());
        ret
    }
}

pub struct FnDefQueryMatches<'q, 't> where 't: 'q {
    query: &'q FnDefQuery,
    cursor: Pin<Box<QueryCursor>>,
    matches: QueryMatches<'q, 't, &'t [u8], &'t [u8]>,
}

pub struct FnDefQueryMatch<'t> {
    pub return_t: Vec<InContext<'t, Node<'t>>>,
    pub name: InContext<'t, Node<'t>>,
}

#[cfg(test)]
mod test {
    use crate::LANG;
    use super::*;

    const HEADER: &'static str = include_str!("test/dummy.hpp");

    fn trad_query() {
        let context = Context::try_from_buffer(HEADER).expect("Parse error");
        let root = context.root();

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&FN_DEF_QUERY.query, root.item, context.text());
        while let Some(m) = matches.next() {
            for node in m.nodes_for_capture_index(FN_DEF_QUERY.name) {
                let node = InContext::new(node, &context);
            }
        }
    }

    fn nova_query() {
        let context = Context::try_from_buffer(HEADER).expect("Parse error");
        let root = context.root();

        let mut matches: FnDefQueryMatches = FN_DEF_QUERY.matches(root);
        while let Some(m) = matches.next() {
            for node: InContext<Node> in m.name() {
                // do something
            }
        }
    }
}