use std::sync::LazyLock;
use tree_sitter::{Node, Query, QueryCursor, StreamingIterator};
use crate::{Context, InContext};

pub static FN_DEF_QUERY: LazyLock<FnDefQuery> = LazyLock::new(|| {
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

pub type QueryMatches<'t, 'q> = tree_sitter::QueryMatches<'q, 't, &'t [u8], &'t [u8]>;

#[derive(Debug)]
pub struct FnDefQuery {
    pub query: Query,
    pub return_t: u32,
    pub name: u32,
    // pub ns: u32,
    // pub arg_t: u32,
    // pub arg_name: u32,
}

pub struct FnDefQueryMatches<'t, 'q> {
    context: &'t Context,
    query: &'q FnDefQuery,
    matches: QueryMatches<'t, 'q>,
    current: Option<FnDefQueryMatch<'t>>,
}

#[derive(Debug, Clone)]
pub struct FnDefQueryMatch<'t> {
    pub return_t: InContext<'t, Node<'t>>,
    pub name: InContext<'t, Node<'t>>,
}

impl<'t> InContext<'t, QueryCursor> {
    pub fn matches<'c, 'q>(
        &'c mut self, 
        query: &'q FnDefQuery, 
        node: impl Into<Node<'t>>,
    ) -> FnDefQueryMatches<'t, 'q>
    where 't: 'q, 'c: 'q {
        FnDefQueryMatches {
            matches: self.item.matches(&query.query, node.into(), self.context.text()), 
            context: self.context,
            query,
            current: None,
        }
    }
}

impl<'t, 'q> StreamingIterator for FnDefQueryMatches<'t, 'q>
where 't: 'q {
    type Item = FnDefQueryMatch<'t>;

    fn advance(&mut self) {
        self.matches.advance();
        self.current = self.matches.get().map(|m| {
            FnDefQueryMatch {
                return_t: self.context.wrap(
                    m.nodes_for_capture_index(self.query.return_t).next().unwrap(),
                ),
                name: self.context.wrap(
                    m.nodes_for_capture_index(self.query.name).next().unwrap(),
                ),
            }
        });
    }

    fn get(&self) -> Option<&Self::Item> {
        self.current.as_ref()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const HEADER: &'static str = r#"
        namespace ns {
            class MyClass {
            public:
                MyClass(int arg1);
                ~MyClass();
                
                const void* pub_method(int arg1, float arg2);
        
            private:
                int priv_method(char* arg1, uint8_t* arg2);
            };
        }
    "#;

    #[derive(Debug, Clone, Copy)]
    struct MatchLiteral {
        return_t: &'static str,
        name: &'static str,
    }
    impl PartialEq<FnDefQueryMatch<'_>> for MatchLiteral {
        fn eq(&self, other: &FnDefQueryMatch) -> bool {
            other.return_t == self.return_t
                && other.name == self.name
        }
    }

    #[test]
    fn query() {
        let context = Context::try_from_buffer(HEADER).expect("Parse error");
        let root = context.root();

        let mut cursor = context.wrap(QueryCursor::new());
        let mut matches = cursor.matches(&FN_DEF_QUERY, root);

        let mut ms = vec![];
        while let Some(m) = matches.next() {
            ms.push(m.clone());
        }
    }
}
