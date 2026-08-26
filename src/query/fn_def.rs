use crate::query::structured_query;

structured_query!(pub FN_DEF_QUERY / FnDefQuery / FnDefMatch / FnDefMatchTest {
    return_q: Option,
    return_t: _,
    return_r: Option,
    name: _,
    args: _,
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
                        parameters: (parameter_list) @args
                    )
                )
                (function_declarator
                    declarator: (field_identifier) @name
                    parameters: (parameter_list) @args
                )
            ]
        ) @root
    )
"#);

#[cfg(test)]
mod test {
    use super::*;
    use crate::{Context, print_node};

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
        let expected = [FnDefMatchTest {
            return_q: Some("const"),
            return_t: "Session",
            return_r: Some("&"),
            name: "my_method",
            args: "(int simple_arg, const std::unique_ptr<Class>& complex_arg)",
            root: "const Session& my_method(int simple_arg, const std::unique_ptr<Class>& complex_arg);",
        }];

        let ctx = Context::from_buffer(header).expect("Parse error");
        let root = ctx.root();
        print_node(root, 10, 50, true);

        crate::query::test::collection_eq(FN_DEF_QUERY.matches(root), expected);
    }
}
