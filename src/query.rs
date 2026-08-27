use crate::InContext;
use std::fmt::Display;
use tree_sitter::{Node, QueryError};

pub(crate) mod fn_arg;
pub(crate) mod fn_def;
pub(crate) mod typeref;

pub trait StructuredQuery: Sized {
    type Match<'tree>: StructuredMatch<'tree>;
    type MatchLiteral<'tree>: PartialEq<Self::Match<'tree>>;

    fn inst() -> &'static Self;
    fn compile() -> Result<Self, StructuredQueryError>;
}

pub trait StructuredMatch<'tree>: Sized {
    type Query: StructuredQuery;

    fn from_node(node: InContext<'tree, Node<'tree>>) -> Vec<Self>;
    fn depth_from(&self, other: Node<'tree>) -> usize;
}

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

macro_rules! structured_query (
    (
        $vis:vis $static_name:ident / $query_name:ident / $match_name:ident / $test_name:ident {
            $( $field:ident: $field_t:tt ),+ $(,)?
        },
        $query_text:literal
    ) => {
        static $static_name: std::sync::LazyLock<$query_name> = std::sync::LazyLock::new(|| {
            use $crate::query::StructuredQuery;
            $query_name::compile().expect(concat!("Failed to compile query", stringify!($query_name)))
        });

        $vis struct $query_name {
            query: ::tree_sitter::Query,
            root: u32,
            $( $field: u32, )*
        }

        impl $crate::query::StructuredQuery for $query_name {
            type Match<'tree> = $match_name<'tree>;
            type MatchLiteral<'tree> = $test_name<'tree>;

            fn inst() -> &'static Self {
                &$static_name
            }

            fn compile() -> Result<Self, $crate::query::StructuredQueryError> {
                let q = ::tree_sitter::Query::new(&$crate::LANG, $query_text)?;
                Ok(Self {
                    root: q.capture_index_for_name("root")
                        .ok_or_else(|| $crate::query::StructuredQueryError::Capture("root"))?,
                    $(
                        $field: q.capture_index_for_name(stringify!($field))
                            .ok_or_else(|| $crate::query::StructuredQueryError::Capture(stringify!($field)))?
                    , )*
                    query: q,
                })
            }
        }

        macro_rules! field_t (
            (_) => {
                $crate::InContext<'tree, ::tree_sitter::Node<'tree>>
            };
            (Option) => {
                Option<$crate::InContext<'tree, ::tree_sitter::Node<'tree>>>
            };
            ([$sub_q:ty]) => {
                Vec<<$sub_q as $crate::query::StructuredQuery>::Match<'tree>>
            };
            ($sub_q:ty) => {
                <$sub_q as $crate::query::StructuredQuery>::Match<'tree>
            };
        );

        macro_rules! field_val (
            ($query:expr, $ctx:ident, $m:ident, $f:ident, _) => {
                $ctx.wrap($m.nodes_for_capture_index($query.$f).next().unwrap())
            };
            ($query:expr, $ctx:ident, $m:ident, $f:ident, Option) => {
                $m.nodes_for_capture_index($query.$f).next().map(|n| $ctx.wrap(n))
            };
            ($query:expr, $ctx:ident, $m:ident, $f:ident, [$sub_q:ty]) => {
                $m.nodes_for_capture_index($query.$f).map(|n| {
                    <
                        <$sub_q as $crate::query::StructuredQuery>
                            ::Match<'tree> as $crate::query::StructuredMatch
                    >::from_node(
                        $ctx.wrap(n),
                    ).into_iter()
                }).flatten().collect()
            };
            ($query:expr, $ctx:ident, $m:ident, $f:ident, $sub_q:ty) => {
                <
                    <$sub_q as $crate::query::StructuredQuery>
                        ::Match<'tree> as $crate::query::StructuredMatch
                >::from_node(
                    $ctx.wrap($m.nodes_for_capture_index($query.$f).next().unwrap()),
                ).into_iter().next().unwrap()
            };
        );

        #[derive(Clone)]
        $vis struct $match_name <'tree> {
            root: ::tree_sitter::Node<'tree>,
            $( $vis $field : field_t!($field_t) ),+
        }

        impl<'tree> $crate::query::StructuredMatch<'tree> for $match_name<'tree> {
            type Query = $query_name;

            fn from_node(node: $crate::InContext<'tree, ::tree_sitter::Node<'tree>>)
                -> Vec<Self> {
                use ::tree_sitter::StreamingIterator;
                use $crate::query::StructuredQuery;

                let mut cursor = ::tree_sitter::QueryCursor::new();
                let mut matches = cursor.matches(&$query_name::inst().query, node.item, node.context.text());
                let mut ret = vec![];

                while let Some(m) = matches.next() {
                    ret.push($match_name::from_match(node.context, m));
                }
                ret.sort_by(|a, b| {
                    <$match_name as $crate::query::StructuredMatch>::depth_from(b, node.item)
                        .cmp(&<$match_name as $crate::query::StructuredMatch>::depth_from(a, node.item))
                });

                ret
            }

            fn depth_from(&self, other: ::tree_sitter::Node<'tree>) -> usize {
                let mut d = 0;
                let mut parent = self.root;
                while let Some(p) = parent.parent() {
                    d += 1;
                    if (p == other) {
                        return d;
                    } else {
                        parent = p;
                    }
                }
                usize::MAX
            }
        }

        impl<'tree> $match_name<'tree> {
            fn from_match(ctx: &'tree $crate::Context, m: &::tree_sitter::QueryMatch<'_, 'tree>) -> Self {
                use $crate::query::StructuredQuery;

                Self {
                    root: m.nodes_for_capture_index($query_name::inst().root).next().unwrap(),
                    $( $field: field_val!($query_name::inst(), ctx, m, $field, $field_t) ),+
                }
            }
        }

        impl std::fmt::Debug for $match_name<'_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{} {{ ", stringify!($match_name))?;
                $( write!(f, "{}: {:?}, ", stringify!($field), self.$field)?; )+
                Ok(())
            }
        }

        macro_rules! test_t (
            (_) => {
                &'tree str
            };
            (Option) => {
                Option<&'tree str>
            };
            ([$sub_q:ty]) => {
                Vec<<$sub_q as $crate::query::StructuredQuery>::MatchLiteral<'tree>>
            };
            ($sub_q:ty) => {
                <$sub_q as $crate::query::StructuredQuery>::MatchLiteral<'tree>
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
            {$self:ident, $other:ident, $f:ident, [$sub_q:ty]} => {
                $self.$f.iter().zip($other.$f.iter()).all(|(s, o)| s == o)
            };
            {$self:ident, $other:ident, $f:ident, $sub_q:ty} => {
                $self.$f == $other.$f
            };
        );

        #[allow(unused)]
        #[derive(Debug, Clone, PartialEq, Eq, std::hash::Hash)]
        $vis struct $test_name <'tree> {
            $( $vis $field : test_t!($field_t) ),+
        }

        impl<'tree> PartialEq<$match_name<'tree>> for $test_name<'tree> {
            fn eq(&self, other: &$match_name<'tree>) -> bool {
                [
                    $( test_eq!(self, other, $field, $field_t) ),+
                ].into_iter().all(|v| v)
            }
        }
    };
);
pub(crate) use structured_query;

#[cfg(test)]
pub mod test {
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
}
