use std::fmt::Display;
use tree_sitter::QueryError;

mod fn_arg;
mod fn_def;

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
        $vis struct $query_name {
            query: ::tree_sitter::Query,
            $( $field: u32, )*
        }

        impl $query_name {
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

            pub fn matches<'tree>(&self, node: $crate::InContext<'tree, ::tree_sitter::Node<'tree>>)
                -> impl ExactSizeIterator<Item = $match_name<'tree>> {
                use ::tree_sitter::StreamingIterator;
                let mut cursor = ::tree_sitter::QueryCursor::new();
                let mut matches = cursor.matches(&self.query, node.item, node.context.text());
                let mut ret = vec![];

                while let Some(m) = matches.next() {
                    ret.push($match_name::from_match(self, node.context, m))
                }

                ret.into_iter()
            }
        }

        macro_rules! field_t (
            (_) => {
                $crate::InContext<'tree, ::tree_sitter::Node<'tree>>
            };
            (Option) => {
                Option<$crate::InContext<'tree, ::tree_sitter::Node<'tree>>>
            };
        );

        macro_rules! field_val (
            ($query:ident, $ctx:ident, $m:ident, $f:ident, _) => {
                $ctx.wrap($m.nodes_for_capture_index($query.$f).next().unwrap())
            };
            ($query:ident, $ctx:ident, $m:ident, $f:ident, Option) => {
                $m.nodes_for_capture_index($query.$f).next().map(|n| $ctx.wrap(n))
            };
        );

        #[derive(Debug, Clone)]
        $vis struct $match_name <'tree> {
            $( $field : field_t!($field_t) ),+
        }

        impl<'tree> $match_name<'tree> {
            pub fn from_match(query: &$query_name, ctx: &'tree $crate::Context, m: &::tree_sitter::QueryMatch<'_, 'tree>) -> Self {
                Self {
                    $( $field: field_val!(query, ctx, m, $field, $field_t) ),+
                }
            }
        }

        $vis static $static_name: std::sync::LazyLock<$query_name> = std::sync::LazyLock::new(|| {
            $query_name::compile().expect(concat!("Failed to compile query", stringify!($query_name)))
        });

        macro_rules! test_t (
            (_) => {
                &'tree str
            };
            (Option) => {
                Option<&'tree str>
            };
        );

        macro_rules! test_eq (
            ($self:ident, $other:ident, $f:ident, _) => {
                $other.$f.as_str().is_ok_and(|f| f == $self.$f)
            };
            ($self:ident, $other:ident, $f:ident, Option) => {
                (match ($other.$f, $self.$f) {
                    (Some(o), Some(s)) => o.as_str().is_ok_and(|f| f == s),
                    (None, None) => true,
                    _ => false,
                })
            };
        );

        #[allow(unused)]
        #[derive(Debug, Clone, PartialEq, Eq, std::hash::Hash)]
        $vis struct $test_name <'tree> {
            $( $field : test_t!($field_t) ),+
        }

        impl<'tree> PartialEq<$match_name<'tree>> for $test_name<'tree> {
            fn eq(&self, other: &$match_name) -> bool {
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
    use crate::Context;
    use std::{collections::HashSet, fmt::Debug, hash::Hash};

    pub fn collection_eq<A, E>(
        actual: impl Iterator<Item = A>,
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

    structured_query!(pub TEST_QUERY / TestQuery / TestMatch / TestMatchLiteral {
        first: _,
        second: Option,
    }, r#"
        (
            (_) @first
            (_) @second
            (_) @third
        )
    "#);

    const CODE: &'static str = r#"
        class MyClass {
        public:
            const Session& my_method(int arg1, const Config& arg2) const;
        };
    "#;

    #[test]
    fn macro_test() {
        let ctx = Context::from_buffer(CODE).expect("Failed to parse code snippet");
        let root = ctx.root();
        let query = TEST_QUERY.matches(root);
    }
}
