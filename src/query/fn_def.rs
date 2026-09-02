use crate::query::structured_query;

structured_query! {
    name: pub FnDef,
    fields: {
        ret: Typeref,
        decl: FnDecl,
    },
    query: r#"
        (field_declaration_list
            ((access_specifier) @variable.builtin (#eq? @variable.builtin "public"))
            .
            (field_declaration)*
            .
            (field_declaration
                declarator: (_) @decl
            ) @ret
        )
    "#
}

structured_query! {
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

structured_query! {
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
        ) @typ
    "#
}

structured_query! {
    name: pub Typeref,
    fields: {
        q: Option,
        t: _,
        r: [Ref],
    },
    query: r#"
        (_
            (type_qualifier ("const"))? @q
            type: [(primitive_type) (type_identifier) (qualified_identifier)] @t
            declarator: (_) @r
        )
    "#,
}

structured_query! {
    name: pub Ref,
    fields: {
        r: _,
        q: Option,
    },
    query: r#"
        [
            (reference_declarator
                [("&") ("&&")] @r
                (identifier)
            )
            (pointer_declarator
                ("*") @r
                (type_qualifier ("const"))? @q
                declarator: [(pointer_declarator) (identifier)]
            )
        ]
    "#,
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{Context, print_node};

    #[test]
    fn def() {
        let header = r#"
            class MyClass {
            public:
                int my_field;
                const Session& my_method(int simple_arg, const std::unique_ptr<Class>& complex_arg) const noexcept;
                void second_method();
            private:
                void internal_method();
            };
        "#;
        let expected = [
            FnDef::Literal {
                ret: Typeref::Literal {
                    q: Some("const"),
                    t: "Session",
                    r: vec![Ref::Literal { r: "&", q: None }],
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
                                r: vec![Ref::Literal { r: "&", q: None }],
                            },
                            name: "complex_arg",
                        },
                    ],
                    immut: Some("const"),
                    noexcept: Some("noexcept"),
                },
            },
            FnDef::Literal {
                ret: Typeref::Literal {
                    q: None,
                    t: "void",
                    r: vec![],
                },
                decl: FnDecl::Literal {
                    name: "second_method",
                    args: vec![],
                    immut: None,
                    noexcept: None,
                },
            },
        ];

        let ctx = Context::from_buffer(header).expect("Parse error");
        let root = ctx.root();
        print_node(root, 10, 50, true);

        crate::query::test::collection_eq(FnDef::Match::from_node(root), expected);
    }

    #[test]
    fn arg() {
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
                    r: vec![Ref::Literal { r: "&", q: None }],
                },
                name: "complex_arg",
            },
        ];

        let ctx = Context::from_buffer(code).expect("Parse error");
        let root = ctx.root();
        print_node(root, 10, 50, true);

        crate::query::test::collection_eq(FnArg::Match::from_node(root), expected);
    }

    #[test]
    fn simple() {
        let code = "int x = 0;";
        let expected = [Typeref::Literal {
            q: None,
            t: "int",
            r: vec![],
        }];

        let ctx = Context::from_buffer(code).expect("Failed to parse snippet");
        let root = ctx.root();
        print_node(root, 15, 50, true);
        super::super::test::collection_eq(Typeref::Match::from_node(root), expected);
    }

    #[test]
    fn ref1() {
        let code = "Session& x = y;";
        let expected = [Typeref::Literal {
            q: None,
            t: "Session",
            r: vec![Ref::Literal { r: "&", q: None }],
        }];

        let ctx = Context::from_buffer(code).expect("Failed to parse snippet");
        let root = ctx.root();
        print_node(root, 15, 50, true);
        super::super::test::collection_eq(Typeref::Match::from_node(root), expected);
    }

    #[test]
    fn ref2() {
        let code = "Session&& x = y;";
        let expected = [Typeref::Literal {
            q: None,
            t: "Session",
            r: vec![Ref::Literal { r: "&&", q: None }],
        }];

        let ctx = Context::from_buffer(code).expect("Failed to parse snippet");
        let root = ctx.root();
        print_node(root, 15, 50, true);
        super::super::test::collection_eq(Typeref::Match::from_node(root), expected);
    }

    #[test]
    fn ptr1() {
        let code = "Session* x = y;";
        let expected = [Typeref::Literal {
            q: None,
            t: "Session",
            r: vec![Ref::Literal { r: "*", q: None }],
        }];

        let ctx = Context::from_buffer(code).expect("Failed to parse snippet");
        let root = ctx.root();
        print_node(root, 15, 50, true);
        super::super::test::collection_eq(Typeref::Match::from_node(root), expected);
    }

    #[test]
    fn ptr2() {
        let code = "Session** x = y;";
        let expected = [Typeref::Literal {
            q: None,
            t: "Session",
            r: vec![
                Ref::Literal { r: "*", q: None },
                Ref::Literal { r: "*", q: None },
            ],
        }];

        let ctx = Context::from_buffer(code).expect("Failed to parse snippet");
        let root = ctx.root();
        print_node(root, 15, 50, true);
        super::super::test::collection_eq(Typeref::Match::from_node(root), expected);
    }

    #[test]
    fn ptr3() {
        let code = "Session*** x = y;";
        let expected = [Typeref::Literal {
            q: None,
            t: "Session",
            r: vec![
                Ref::Literal { r: "*", q: None },
                Ref::Literal { r: "*", q: None },
                Ref::Literal { r: "*", q: None },
            ],
        }];

        let ctx = Context::from_buffer(code).expect("Failed to parse snippet");
        let root = ctx.root();
        print_node(root, 15, 50, true);
        super::super::test::collection_eq(Typeref::Match::from_node(root), expected);
    }

    #[test]
    fn ptr_const() {
        let code = "Session* const x = y;";
        let expected = [Typeref::Literal {
            q: None,
            t: "Session",
            r: vec![Ref::Literal {
                r: "*",
                q: Some("const"),
            }],
        }];

        let ctx = Context::from_buffer(code).expect("Failed to parse snippet");
        let root = ctx.root();
        print_node(root, 15, 50, true);
        super::super::test::collection_eq(Typeref::Match::from_node(root), expected);
    }
}
