use tree_sitter::{Node, QueryCursor};
use crate::InContext;

mod fn_def;
pub use fn_def::*;

pub trait StructuredQuery {
    type Match<'t> : StructuredQueryMatch<Self>;
    type Matches<'t> : ExactSizeIterator<Item = Self::Match<'t>>;

    fn eval<'c, 't, 'q>(&'q self, item: &'c mut InContext<'t, QueryCursor>, node: impl Into<Node<'t>>)
        -> Self::Matches<'t>
        where 't: 'q, 'c: 'q;
}

pub trait StructuredQueryMatch<Q> : PartialEq where Q : StructuredQuery + ?Sized {
    type Field : AsRef<str> + ?Sized;

    fn field_value(&self, field_id: u32) -> Option<&Self::Field>;
}

impl<'t> InContext<'t, QueryCursor> {
    pub fn matches<'c, 'q, Q>(
        &'c mut self, 
        query: &'q impl std::ops::Deref<Target = Q>, 
        node: impl Into<Node<'t>>,
    ) -> Q::Matches<'t>
    where 't: 'q, 'c: 'q, Q: StructuredQuery {
        query.eval(self, node)
    }
}
