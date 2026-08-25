use std::sync::LazyLock;
use tree_sitter::{Node, Query, QueryCursor, StreamingIterator};
use crate::{Context, InContext};

pub mod fn_def;

trait StructuredQuery {
    type Match<'t> : StructuredQueryMatch<Self>;
    type Matches<'t> : ExactSizeIterator<Item = Self::Match<'t>>;

    fn eval<'c, 't, 'q>(&'q self, item: &'c mut InContext<'t, QueryCursor>, node: impl Into<Node<'t>>)
        -> Self::Matches<'t>
        where 't: 'q, 'c: 'q;
}

trait StructuredQueryMatch<Q> : PartialEq where Q : StructuredQuery + ?Sized {
    type Field : AsRef<str>;

    fn field_value(&self, field_id: u32) -> Self::Field;
}

impl<'t> InContext<'t, QueryCursor> {
    pub fn matches<'c, 'q, Q>(
        &'c mut self, 
        query: &'q Q, 
        node: impl Into<Node<'t>>,
    ) -> Q::Matches<'t>
    where 't: 'q, 'c: 'q, Q: StructuredQuery {
        query.eval(self, node)
    }
}
