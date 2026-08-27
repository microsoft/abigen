use super::{structured_query, typeref::TyperefQuery};

structured_query!(
    pub FN_ARG_QUERY / FnArgQuery / FnArgMatch / FnArgLitrl {
        typ: TyperefQuery,
        name: _,
    }, r#"
        (parameter_declaration
            declarator: [
                (identifier) @name
                (reference_declarator
                    (identifier) @name
                )
            ]
        ) @typ @root
    "#
);

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        Context, print_node,
        query::{StructuredMatch, typeref::TyperefLitrl},
    };

    #[test]
    fn test() {
        let code =
            "const Session& my_method(int simple_arg, const std::unique_ptr<Class>& complex_arg);";
        let expected = [
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
        ];

        let ctx = Context::from_buffer(code).expect("Parse error");
        let root = ctx.root();
        print_node(root, 10, 50, true);

        crate::query::test::collection_eq(FnArgMatch::from_node(root), expected);
    }
}
