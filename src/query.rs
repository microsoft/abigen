use std::fmt::Display;
use tree_sitter::QueryError;

mod fn_def;
pub(crate) use fn_def::*;

#[derive(Debug)]
pub enum StructuredQueryError {
    Compile(QueryError),
    Capture(&'static str),
}
impl From<QueryError> for StructuredQueryError {
    fn from(value: QueryError) -> Self {
        Self::Compile(value)
    }
}

impl Display for StructuredQueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Compile(e) => write!(f, "Failed to compile query: {e}"),
            Self::Capture(e) => write!(f, "Could not find capture with name: {e}"),
        }
    }
}
impl std::error::Error for StructuredQueryError {}

macro_rules! field_t (
    (_) => {
        $crate::ContextNode<'tree>
    };
    (Option) => {
        Option<$crate::ContextNode<'tree>>
    };
    ([$($sub_q:ident)::+]) => {
        Vec<$($sub_q)::+::Match<'tree>>
    };
    ($($sub_q:ident)::+) => {
        $($sub_q)::+::Match<'tree>
    };
);

macro_rules! field_val (
    ($query:expr, $ctx:ident, $m:ident, $f:ident, _) => {
        $m.nodes_for_capture_index($query.$f).next().map(|n| $ctx.wrap(n))
    };
    ($query:expr, $ctx:ident, $m:ident, $f:ident, Option) => {
        Some($m.nodes_for_capture_index($query.$f).next().map(|n| $ctx.wrap(n)))
    };
    ($query:expr, $ctx:ident, $m:ident, $f:ident, [$($sub_q:ident)::+]) => {
        Some($m.nodes_for_capture_index($query.$f).map(|n| {
            $($sub_q)::+::Match::from_node(
                $ctx.wrap(n),
            ).into_iter()
        }).flatten().collect())
    };
    ($query:expr, $ctx:ident, $m:ident, $f:ident, $($sub_q:ident)::+) => {
        $($sub_q)::+::Match::from_node(
            $ctx.wrap($m.nodes_for_capture_index($query.$f).next().unwrap()),
        ).into_iter().next()
    };
);

macro_rules! test_t (
    (_) => {
        &'tree str
    };
    (Option) => {
        Option<&'tree str>
    };
    ([$($sub_q:ident)::+]) => {
        Vec<$($sub_q)::+::Literal<'tree>>
    };
    ($($sub_q:ident)::+) => {
        $($sub_q)::+::Literal<'tree>
    };
);

macro_rules! test_eq (
    ($self:ident, $other:ident, $f:ident, _) => {
        $other.$f.as_str().is_ok_and(|f| f == $self.$f)
    };
    ($self:ident, $other:ident, $f:ident, Option) => {
        (match ($self.$f, $other.$f) {
            (Some(s), Some(o)) => o.as_str().is_ok_and(|f| f == s),
            (None, None) => true,
            _ => false,
        })
    };
    ($self:ident, $other:ident, $f:ident, [$($sub_q:ident)::+]) => {
        $self.$f.len() == $other.$f.len()
            && $self.$f.iter().zip($other.$f.iter()).all(|(s, o)| s == o)
    };
    ($self:ident, $other:ident, $f:ident, $($sub_q:ident)::+) => {
        $self.$f == $other.$f
    };
);

macro_rules! structured_query {
(
    name: $vis:vis $name:ident,
    fields: {
        $( $field:ident: $field_t:tt ),+ $(,)?
    },
    query: $query_text:literal $(,)?
) => {
    #[allow(non_snake_case)]
    $vis mod $name {
        #[allow(unused)]
        use super::*;

        pub static QUERY_INST: std::sync::LazyLock<Query> = std::sync::LazyLock::new(|| {
            Query::compile().expect(concat!("Failed to compile query ", stringify!($name)))
        });

        pub struct Query {
            query: ::tree_sitter::Query,
            $( $field: u32, )*
        }

        impl Query {
            pub fn compile() -> Result<Self, $crate::query::StructuredQueryError> {
                let q = ::tree_sitter::Query::new(&$crate::LANG, $query_text)?;
                Ok(Self {
                    $(
                        $field: q.capture_index_for_name(stringify!($field))
                            .ok_or_else(|| $crate::query::StructuredQueryError::Capture(stringify!($field)))?
                    , )*
                    query: q,
                })
            }
        }

        #[derive(Clone, std::hash::Hash)]
        pub struct Match<'tree> {
            $( pub $field : $crate::query::field_t!($field_t), )+
            byte_range: std::ops::Range<usize>,
        }

        impl<'tree> Match<'tree> {
            pub fn from_node(node: $crate::ContextNode<'tree>)
                -> Vec<Self> {
                use ::tree_sitter::StreamingIterator;

                let mut cursor = ::tree_sitter::QueryCursor::new();
                let mut matches = cursor.matches(&QUERY_INST.query, node.node, node.context.text());
                let mut ret = vec![];

                while let Some(m) = matches.next() {
                    if let Some(m) = Match::from_match(node.context, m) {
                        ret.push(m);
                    }
                }
                ret.sort_by(|a, b| {
                    a.byte_range.start.cmp(&b.byte_range.start)
                        .then(a.byte_range.end.cmp(&b.byte_range.end).reverse())
                });

                ret
            }

            fn from_match(ctx: &'tree $crate::Context, m: &::tree_sitter::QueryMatch<'_, 'tree>) -> Option<Self> {
                let byte_range = m.captures.iter()
                    .fold(None, |acc, c| {
                        let acc = acc.unwrap_or(c.node.byte_range());
                        Some(acc.start.min(c.node.start_byte()) .. acc.end.max(c.node.end_byte()))
                    }).unwrap();
                Some(Self {
                    $( $field: $crate::query::field_val!(QUERY_INST, ctx, m, $field, $field_t)?, )+
                    byte_range,
                })
            }
        }

        impl std::fmt::Debug for Match<'_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}::Match {{ ", stringify!($name))?;
                $( write!(f, "{}: {:?}, ", stringify!($field), self.$field)?; )+
                write!(f, "}}")?;
                Ok(())
            }
        }

        #[allow(unused)]
        #[derive(Debug, Clone, PartialEq, Eq, std::hash::Hash)]
        pub struct Literal<'tree> {
            $( pub $field : $crate::query::test_t!($field_t) ),+
        }

        impl<'tree> PartialEq<Match<'tree>> for Literal<'tree> {
            fn eq(&self, other: &Match<'tree>) -> bool {
                [
                    $( $crate::query::test_eq!(self, other, $field, $field_t) ),+
                ].into_iter().all(|v| v)
            }
        }
    }
};
}
pub(crate) use {field_t, field_val, structured_query, test_eq, test_t};

#[cfg(test)]
pub mod test {
    use crate::{Context, print_node};
    use std::{collections::HashSet, fmt::Debug, hash::Hash};

    pub fn collection_eq<A, E>(
        actual: impl IntoIterator<Item = A>,
        expected: impl IntoIterator<Item = E>,
    ) where
        A: Debug,
        E: Debug + PartialEq + Eq + Hash + PartialEq<A>,
    {
        let mut expected: HashSet<_> = expected.into_iter().collect();

        let mut unexpected = vec![];
        for item in actual {
            let expected = expected.extract_if(|i| i == &item).next();
            if expected.is_none() {
                unexpected.push(item);
            }
        }
        let unmatched: Vec<_> = expected.into_iter().collect();

        assert_eq!(unmatched, unexpected, "expected (left) != actual (right)");
    }

    structured_query! {
        name: pub Q,
        fields: {
            typ: _,
            size: Option,
        },
        query: r#"
            (declaration
                type: (_) @typ
                declarator: (init_declarator
                    declarator: [
                        (identifier)
                        (array_declarator
                            size: (_) @size
                        )
                    ]
                )
            ) @root
        "#,
    }

    #[test]
    fn simple() {
        let code = r#"
            int number = 5;
        "#;
        let expected = vec![Q::Literal {
            typ: "int",
            size: None,
        }];

        let ctx = Context::from_buffer(code).expect("Failed to parse snippet");
        let root = ctx.root();
        print_node(root, 10, 50, true);

        collection_eq(Q::Match::from_node(root), expected);
    }

    #[test]
    fn complex() {
        let code = r#"
            int array[5] = {1, 2, 3, 4, 5};
        "#;
        let expected = vec![Q::Literal {
            typ: "int",
            size: Some("5"),
        }];

        let ctx = Context::from_buffer(code).expect("Failed to parse snippet");
        let root = ctx.root();
        print_node(root, 10, 50, true);

        collection_eq(Q::Match::from_node(root), expected);
    }
}
