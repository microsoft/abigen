use std::fmt::Display;
use tree_sitter::QueryError;

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
        $vis:vis $static_name:ident / $query_name:ident / $match_name:ident {
            $( $field:ident: $field_t:tt ),+ $(,)?
        }
        ,
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

            pub fn matches<'tree>(&self, node: $crate::InContext<'tree, ::tree_sitter::Node<'tree>>) -> impl ExactSizeIterator<Item = $match_name<'tree>> {
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
            (Vec) => {
                Vec<$crate::InContext<'tree, ::tree_sitter::Node<'tree>>>
            };
        );

        macro_rules! field_val (
            ($query:ident, $ctx:ident, $m:ident, $f:ident, _) => {
                $ctx.wrap($m.nodes_for_capture_index($query.$f).next().unwrap())
            };
            ($query:ident, $ctx:ident, $m:ident, $f:ident, Option) => {
                $m.nodes_for_capture_index($query.$f).next().map(|n| $ctx.wrap(n))
            };
            ($query:ident, $ctx:ident, $m:ident, $f:ident, Vec) => {
                $m.nodes_for_capture_index($query.$f).map(|n| $ctx.wrap(n)).collect()
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
    };
);
pub(crate) use structured_query;

#[cfg(test)]
mod test {
    use crate::Context;

    structured_query!(pub FN_DEF_QUERY / FnDefQuery / FnDefMatch {
        first: _,
        second: Option,
        third: Vec,
    }, r#"
        (_) @first
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
        let query = FN_DEF_QUERY.matches(root);
    }
}
