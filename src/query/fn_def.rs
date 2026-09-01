use crate::query::{fn_arg::FnArg, structured_query, typeref::Typeref};

structured_query!{
    name: pub FnDef,
    fields: {
        ret: Typeref,
        decl: FnDecl,
    },
    query: r#"
        (field_declaration_list
            ((access_specifier) @variable.builtin (#eq? @variable.builtin "public"))
            .
            (field_declaration
                declarator: (_) @decl
            ) @ret
        )
    "#
}

structured_query!{
    name: pub FnDecl,
    fields: {
        name: _,
        args: [FnArg],
        immut: Option,
        noexcept: Option,
    },
    query: r#"
        (function_declarator
            declarator: (field_identifier) @name
            parameters: (parameter_list) @args
            (type_qualifier ("const") @immut)?
            (noexcept ("noexcept") @noexcept)?
        )
    "#
}

#[cfg(test)]
mod test {
    use std::vec;

use super::*;
    use crate::{
        Context, print_node, query::{StructuredMatch, typeref::{Ref, Typeref}},
    };
    use super::{FnArg, FnDecl};

    #[test]
    fn test() {
        let header = r#"
            class MyClass {
            public:
                const Session& my_method(int simple_arg, const std::unique_ptr<Class>& complex_arg) const noexcept;
            private:
                void internal_method();
            };
        "#;
        let expected = [FnDef::Literal {
            ret: Typeref::Literal {
                q: Some("const"),
                t: "Session",
                r: vec![Ref::Literal { r: "&" }],
            },
            decl: FnDecl::Literal {
                name: "my_method",
                args: vec![
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
                ],
                immut: Some("const"),
                noexcept: Some("noexcept"),
            },
        }];

        let ctx = Context::from_buffer(header).expect("Parse error");
        let root = ctx.root();
        print_node(root, 10, 50, true);

        crate::query::test::collection_eq(FnDef::Match::from_node(root), expected);
    }
}
