use crate::query::structured_query;

structured_query!(pub FN_DEF_QUERY / FnDefQuery / FnDefMatch {
    return_q: Option,
    return_t: _,
    return_r: Option,
    name: _,
    root: _,
}, r#"
    (field_declaration_list
        ((access_specifier) @variable.builtin (#eq? @variable.builtin "public"))
        .
        (field_declaration
            (type_qualifier)? @return_q
            type: (type_identifier) @return_t
            declarator: [
                (reference_declarator
                    ("&") @return_r
                    (function_declarator
                        declarator: (field_identifier) @name
                    )
                )
                (function_declarator
                    declarator: (field_identifier) @name
                )
            ]
        ) @root
    )
"#);

#[cfg(test)]
mod test {
    use std::{collections::HashSet, fmt::Debug, hash::Hash};
    use crate::{Context, print_node, query::fn_def::{FN_DEF_QUERY, FnDefMatch}};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    struct MatchLiteral {
        return_q: Option<&'static str>,
        return_t: &'static str,
        return_r: Option<&'static str>,
        name: &'static str,
        root: &'static str,
    }

    impl PartialEq<FnDefMatch<'_>> for MatchLiteral {
        fn eq(&self, other: &FnDefMatch) -> bool {
            let Self {
                return_q, return_t, return_r, name, ..
            } = self;
            true
            && match (other.return_q, return_q) {
                (Some(orq), Some(srq)) => orq.as_ref() == *srq,
                (None, None) => true,
                _ => false,
            }
            && other.return_t.as_str().is_ok_and(|t| t == *return_t)
            && other.return_r.map(|r| r.as_str().ok()).flatten() == *return_r
            && other.name.as_str().is_ok_and(|t| t == *name)
        }
    }

    fn collection_eq<'a>(actual: impl Iterator<Item = FnDefMatch<'a>>, expected: impl IntoIterator<Item = MatchLiteral>){
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
        let expected = [
            MatchLiteral {
                return_q: Some("const"),
                return_t: "Session",
                return_r: Some("&"),
                name: "my_method",
                root: header,
            },
        ];

        let ctx = Context::from_buffer(header).expect("Parse error");
        let root = ctx.root();
        print_node(root, 10, 50, true);

        collection_eq(FN_DEF_QUERY.matches(root), expected);
    }
}