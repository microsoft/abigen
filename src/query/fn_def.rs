use crate::query::{fn_arg::FnArgQuery, structured_query, typeref::TyperefQuery};

structured_query!(pub FN_DEF_QUERY / FnDefQuery / FnDefMatch / FnDefLitrl {
    ret: TyperefQuery,
    name: _,
    args: [FnArgQuery],
}, r#"
    (field_declaration_list
        ((access_specifier) @variable.builtin (#eq? @variable.builtin "public"))
        .
        (field_declaration
            declarator: [
                (function_declarator
                    declarator: (field_identifier) @name
                    parameters: (parameter_list) @args
                )
                (_
                    (function_declarator
                        declarator: (field_identifier) @name
                        parameters: (parameter_list) @args
                    )
                )
            ]
        ) @ret @root
    )
"#);

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        Context, print_node,
        query::{StructuredMatch, fn_arg::FnArgLitrl, typeref::TyperefLitrl},
    };

    #[test]
    fn test() {
        let header = r#"
            class MyClass {
            public:
                const Session& my_method(int simple_arg, const std::unique_ptr<Class>& complex_arg);
            private:
                void internal_method();
            };
        "#;
        let expected = [FnDefLitrl {
            ret: TyperefLitrl {
                q: Some("const"),
                t: "Session",
                r: Some("&"),
            },
            name: "my_method",
            args: vec![
                FnArgLitrl {
                    typ: TyperefLitrl {
                        q: None,
                        t: "int",
                        r: None,
                    },
                    name: "simple_arg",
                },
                FnArgLitrl {
                    typ: TyperefLitrl {
                        q: Some("const"),
                        t: "std::unique_ptr<Class>",
                        r: Some("&"),
                    },
                    name: "complex_arg",
                },
            ],
        }];

        let ctx = Context::from_buffer(header).expect("Parse error");
        let root = ctx.root();
        print_node(root, 10, 50, true);

        crate::query::test::collection_eq(FnDefMatch::from_node(root), expected);
    }
}
