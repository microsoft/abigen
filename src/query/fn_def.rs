use std::{fmt::Debug, sync::LazyLock};
use tree_sitter::{Node, Query, QueryCursor, StreamingIterator};
use crate::{InContext, query::{StructuredQuery, StructuredQueryMatch}};

pub static FN_DEF_QUERY: LazyLock<FnDefQuery> = LazyLock::new(|| {
    let query = Query::new(&crate::LANG, r#"
        (_
            type: (_) @return_t
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

#[derive(Debug)]
pub struct FnDefQuery {
    pub query: Query,
    pub return_t: u32,
    pub name: u32,
    // pub ns: u32,
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
            ret.push(FnDefQueryMatch {
                return_t: item.context.wrap(
                    m.nodes_for_capture_index(self.return_t).next().unwrap(),
                ),
                name: item.context.wrap(
                    m.nodes_for_capture_index(self.name).next().unwrap(),
                ),
            })
        }
        ret.into_iter()
    }
}

#[derive(Debug, Clone)]
pub struct FnDefQueryMatch<'t> {
    pub return_t: InContext<'t, Node<'t>>,
    pub name: InContext<'t, Node<'t>>,
}

impl<T> PartialEq<T> for FnDefQueryMatch<'_> where T: StructuredQueryMatch<FnDefQuery> {
    fn eq(&self, other: &T) -> bool {
        self.return_t.as_ref() == other.field_value(FN_DEF_QUERY.return_t).as_ref()
            && self.name.as_ref() == other.field_value(FN_DEF_QUERY.name).as_ref()
    }
}

impl<'t> StructuredQueryMatch<FnDefQuery> for FnDefQueryMatch<'t> {
    type Field = InContext<'t, Node<'t>>;

    fn field_value(&self, field_id: u32) -> &Self::Field {
        if field_id == FN_DEF_QUERY.return_t {
            &self.return_t
        } else if field_id == FN_DEF_QUERY.name {
            &self.name
        } else {
            panic!("Requesting non-existent field!")
        }
    }
}

#[cfg(test)]
mod test {
    use std::{collections::HashSet, fmt::Debug, hash::{Hash, RandomState}};
    use tree_sitter::QueryCursor;
    use crate::{Context, query::{FN_DEF_QUERY, FnDefQuery, StructuredQuery, StructuredQueryMatch}};

    #[derive(Debug, Clone, Copy, Eq, Hash)]
    struct MatchLiteral {
        return_t: &'static str,
        name: &'static str,
    }
    impl<T> PartialEq<T> for MatchLiteral where T: StructuredQueryMatch<FnDefQuery> {
        fn eq(&self, other: &T) -> bool {
            self.return_t == other.field_value(FN_DEF_QUERY.return_t).as_ref()
            && self.name == other.field_value(FN_DEF_QUERY.name).as_ref()
        }
    }
    impl StructuredQueryMatch<FnDefQuery> for MatchLiteral {
        type Field = str;
        fn field_value(&self, field_id: u32) -> &Self::Field {
            if field_id == FN_DEF_QUERY.return_t {
                self.return_t
            } else if field_id == FN_DEF_QUERY.name {
                self.name
            } else {
                panic!("Invalid field request")
            }
        }
    }

    fn collection_eq<Q, Ia, Ta, Ie, Te>(actual: Ia, expected: Ie)
    where Q: StructuredQuery,
        Ta: StructuredQueryMatch<Q> + Debug,
        Te: StructuredQueryMatch<Q> + PartialEq<Ta> + Hash + Eq + Debug,
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

        assert_eq!(unmatched, unexpected);
    }

    #[test]
    fn simple() {
        let header = "const Session& my_method(int simple_arg, const std::unique_ptr<Class>& complex_arg);";
        let expected: HashSet<_, RandomState> = HashSet::from_iter([
            MatchLiteral {
                return_t: "const Session&",
                name: "my_method",
            },
        ]);

        let ctx = Context::try_from_buffer(header).expect("Parse error");
        let root = ctx.root();
        let mut cursor = ctx.wrap(QueryCursor::new());
        collection_eq(cursor.matches(&FN_DEF_QUERY, root), expected);
    }

    // #[test]
    // fn complex() {
    //     let header = r#"
    //         namespace ns {
    //             class MyClass {
    //             public:
    //                 MyClass(int arg1);
    //                 ~MyClass();
                    
    //                 const void* pub_method(int arg1, float arg2);
            
    //             private:
    //                 int priv_method(char* arg1, uint8_t* arg2);
    //             };
    //         }
    //     "#;
    //     let expected = vec![
    //         MatchLiteral {
    //             return_t: "const void*",
    //             name: "pub_method",
    //         },
    //     ];

    //     let ctx = Context::try_from_buffer(header).expect("Parse error");
    //     let root = ctx.root();

    //     let mut cursor = ctx.wrap(QueryCursor::new());
    //     let mut matches = cursor.matches(&FN_DEF_QUERY, root);

    //     let mut i = 0;
    //     while let Some(m) = matches.next() {
    //         assert_eq!(&expected[i], m);
    //         i += 1;
    //     }
    // }
}