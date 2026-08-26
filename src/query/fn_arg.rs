use super::structured_query;

structured_query!(
    pub FN_ARG_QUERY / FnArgQuery / FnArgMatch / FnArgMatchTest {
        arg_q: Option,
        arg_t: _,
        arg_name: _,
        arg_r: Option,
    }, r#"
        (parameter_declaration
            (type_qualifier)? @arg_q
            type: (_) @arg_t
            declarator: [
                (identifier) @arg_name
                (reference_declarator
                    ("&")? @arg_r
                    (identifier) @arg_name
                )
            ]
        )
    "#
);

#[cfg(test)]
mod test {
    use super::*;
    use crate::{Context, print_node};

    #[test]
    fn test() {
        let code =
            "const Session& my_method(int simple_arg, const std::unique_ptr<Class>& complex_arg);";
        let expected = [
            FnArgMatchTest {
                arg_q: None,
                arg_t: "int",
                arg_r: None,
                arg_name: "simple_arg",
            },
            FnArgMatchTest {
                arg_q: Some("const"),
                arg_t: "std::unique_ptr<Class>",
                arg_r: Some("&"),
                arg_name: "complex_arg",
            },
        ];

        let ctx = Context::from_buffer(code).expect("Parse error");
        let root = ctx.root();
        print_node(root, 10, 50, true);

        crate::query::test::collection_eq(FN_ARG_QUERY.matches(root), expected);
    }
}
