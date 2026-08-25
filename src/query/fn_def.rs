use std::{fmt::Debug, sync::LazyLock};
use tree_sitter::{Node, Query, QueryCursor, StreamingIterator};
use crate::{InContext, query::{StructuredQuery, StructuredMatch}};

pub static FN_DEF_QUERY: LazyLock<FnDefQuery> = LazyLock::new(|| {
    let query = Query::new(&crate::LANG, r#"
        (field_declaration_list
            ((access_specifier) @variable.builtin (#eq? @variable.builtin "public"))
            .
            (field_declaration
                (type_qualifier)? @return_q
                type: (type_identifier) @return_t
                declarator: [
                    (reference_declarator
                        (function_declarator
                            declarator: (field_identifier) @name
                        )
                    ) @return_r
                    (function_declarator
                        declarator: (field_identifier) @name
                    )
                ]
            ) @root
        )
    "#).expect("Failed to compile fn_def");

    FnDefQuery {
        root: query.capture_index_for_name("root")
            .expect("Failed to get capture root"),
        return_q: query.capture_index_for_name("return_q")
            .expect("Failed to get capture return_q"),
        return_t: query.capture_index_for_name("return_t")
            .expect("Failed to get capture return_t"),
        return_r: query.capture_index_for_name("return_r")
            .expect("Failed to get capture return_r"),
        name: query.capture_index_for_name("name")
            .expect("Failed to get capture name"),
        query,
    }
});

pub struct FnDefQuery {
    pub query: Query,
    pub root: u32,
    pub return_q: u32,
    pub return_t: u32,
    pub return_r: u32,
    pub name: u32,
    // pub arg_t: u32,
    // pub arg_name: u32,
}

impl StructuredQuery for FnDefQuery {
    type Match<'t> = FnDefQueryMatch<'t>;
    type Matches<'t> = std::vec::IntoIter<FnDefQueryMatch<'t>>;

    fn eval<'c, 't, 'q>(&'q self, item: &'c mut InContext<'t, QueryCursor>, node: impl Into<Node<'t>>)
        -> Self::Matches<'t>
        where 't: 'q, 'c: 'q
    {
        let mut ret = vec![];
        let mut matches = item.item.matches(&self.query, node.into(), item.context.text());
        while let Some(m) = matches.next() {
            ret.push(FnDefQueryMatch::Method {
                root: item.context.wrap(
                    m.nodes_for_capture_index(self.root).next().unwrap()
                ),
                return_q: m.nodes_for_capture_index(self.return_q).next()
                    .map(|q| item.context.wrap(q)),
                return_t: item.context.wrap(
                    m.nodes_for_capture_index(self.return_t).next().unwrap(),
                ),
                return_r: m.nodes_for_capture_index(self.return_r).next()
                    .map(|q| item.context.wrap(q)),
                name: item.context.wrap(
                    m.nodes_for_capture_index(self.name).next().unwrap(),
                ),
            })
        }
        ret.into_iter()
    }
}

#[derive(Debug, Clone)]
pub enum FnDefQueryMatch<'t> {
    Constructor { },
    Method {
        return_q: Option<InContext<'t, Node<'t>>>,
        return_t: InContext<'t, Node<'t>>,
        return_r: Option<InContext<'t, Node<'t>>>,
        name: InContext<'t, Node<'t>>,
        root: InContext<'t, Node<'t>>,
    },
}

impl<T> PartialEq<T> for FnDefQueryMatch<'_> where T: StructuredMatch<FnDefQuery> {
    fn eq(&self, other: &T) -> bool {
        match self {
            Self::Method {
                return_q, return_t, return_r, name, ..
            } => {
                true // not sure why this is necessary but it helps
                && match (other.field_value(FN_DEF_QUERY.return_q), return_q) {
                    (Some(orq), Some(srq)) => orq.as_ref() == srq.as_ref(),
                    (None, None) => true,
                    _ => false,
                }
                && other.field_value(FN_DEF_QUERY.return_t).is_some_and(|r| r.as_ref() == return_t.as_ref())
                && other.field_value(FN_DEF_QUERY.return_r).is_some() == return_r.is_some()
                && other.field_value(FN_DEF_QUERY.name).is_some_and(|n| n.as_ref() == name.as_ref())
            },
            _ => false,
        }
    }
}

impl<'t> StructuredMatch<FnDefQuery> for FnDefQueryMatch<'t> {
    type Field = InContext<'t, Node<'t>>;

    fn field_value(&self, field_id: u32) -> Option<&Self::Field> {
        match self {
            Self::Method { root, .. } if field_id == FN_DEF_QUERY.root => {
                Some(root)
            },
            Self::Method {return_q, ..} if field_id == FN_DEF_QUERY.return_q => {
                return_q.as_ref()
            },
            Self::Method { return_t, .. } if field_id == FN_DEF_QUERY.return_t => {
                Some(return_t)
            },
            Self::Method {return_r, ..} if field_id == FN_DEF_QUERY.return_r => {
                return_r.as_ref()
            },
            Self::Method { name, .. } if field_id == FN_DEF_QUERY.name => {
                Some(name)
            },
            _ => None,
        }
    }
}

#[cfg(test)]
mod test {
    use std::{collections::HashSet, fmt::Debug, hash::{Hash, RandomState}};
    use tree_sitter::QueryCursor;
    use crate::{Context, print_node, query::{FN_DEF_QUERY, FnDefQuery, StructuredQuery, StructuredMatch}};

    #[derive(Debug, Clone, Copy, Eq, Hash)]
    enum MatchLiteral {
        Constructor {},
        Method {
            return_q: Option<&'static str>,
            return_t: &'static str,
            return_r: Option<&'static str>,
            name: &'static str,
            root: &'static str,
        },
    }
    impl<T> PartialEq<T> for MatchLiteral where T: StructuredMatch<FnDefQuery> {
        fn eq(&self, other: &T) -> bool {
            match self {
                Self::Method {
                    return_q, return_t, return_r, name, ..
                } => {
                    true
                    && match (other.field_value(FN_DEF_QUERY.return_q), return_q) {
                        (Some(orq), Some(srq)) => orq.as_ref() == *srq,
                        (None, None) => true,
                        _ => false,
                    }
                    && other.field_value(FN_DEF_QUERY.return_t).is_some_and(|r| r.as_ref() == *return_t)
                    && other.field_value(FN_DEF_QUERY.return_r).is_some() == return_r.is_some()
                    && other.field_value(FN_DEF_QUERY.name).is_some_and(|n| n.as_ref() == *name)
                },
                _ => false,
            }
        }
    }
    impl StructuredMatch<FnDefQuery> for MatchLiteral {
        type Field = str;
        fn field_value(&self, field_id: u32) -> Option<&Self::Field> {
            match self {
                Self::Method { root, .. } if field_id == FN_DEF_QUERY.root => {
                    Some(root)
                },
                Self::Method { return_q, .. } if field_id == FN_DEF_QUERY.return_q => {
                    *return_q
                },
                Self::Method { return_t, .. } if field_id == FN_DEF_QUERY.return_t => {
                    Some(return_t)
                },
                Self::Method { return_r, .. } if field_id == FN_DEF_QUERY.return_r => {
                    *return_r
                },
                Self::Method { name, .. } if field_id == FN_DEF_QUERY.name => {
                    Some(name)
                },
                _ => None,
            }
        }
    }

    fn collection_eq<Q, Ia, Ta, Ie, Te>(actual: Ia, expected: Ie)
    where Q: StructuredQuery,
        Ta: StructuredMatch<Q> + Debug,
        Te: StructuredMatch<Q> + PartialEq<Ta> + Hash + Eq + Debug,
        Ia: IntoIterator<Item = Ta>,
        Ie: IntoIterator<Item = Te> {
        let mut expected: HashSet<Te> = expected.into_iter().collect();

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

    #[test]
    fn simple() {
        let header = r#"
            class MyClass {
            public:
                const Session& my_method(int simple_arg, const std::unique_ptr<Class>& complex_arg);
            private:
                void internal_method();
            };
        "#;
        let expected: HashSet<_, RandomState> = HashSet::from_iter([
            MatchLiteral::Method {
                return_q: Some("const"),
                return_t: "Session",
                return_r: Some("&"),
                name: "my_method",
                root: header,
            },
        ]);

        let ctx = Context::try_from_buffer(header).expect("Parse error");
        let root = ctx.root();
        print_node(root, 10, 50);

        let mut cursor = ctx.wrap(QueryCursor::new());
        collection_eq(cursor.matches(&FN_DEF_QUERY, root), expected);
    }
}