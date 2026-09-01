use super::{structured_query, typeref::Typeref};

structured_query!{
    name: pub FnArg,
    fields: {
        typ: Typeref,
        name: _,
    },
    query: r#"
        (parameter_declaration
            declarator: [
                (identifier) @name
                (reference_declarator
                    (identifier) @name
                )
            ]
        ) @typ @root
    "#
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        Context, print_node, query::{StructuredMatch, typeref::{Typeref, Ref}},
    };

    #[test]
    fn test() {
        let code =
            "const Session& my_method(int simple_arg, const std::unique_ptr<Class>& complex_arg);";
        let expected = [
            FnArg::Literal {
                typ: Typeref::Literal {
                    q: None,
                    t: "int",
                    r: vec![],
                },
                name: "simple_arg",
            },
            FnArg::Literal {
                typ: Typeref::Literal {
                    q: Some("const"),
                    t: "std::unique_ptr<Class>",
                    r: vec![Ref::Literal { r: "&" }],
                },
                name: "complex_arg",
            },
        ];

        let ctx = Context::from_buffer(code).expect("Parse error");
        let root = ctx.root();
        print_node(root, 10, 50, true);

        crate::query::test::collection_eq(FnArg::Match::from_node(root), expected);
    }
}
