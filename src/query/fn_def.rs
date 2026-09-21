use crate::query::structured_query;

pub mod k {
    macro_rules! kind_id (
        ($vis:vis $name:ident, $kind:literal) => {
            $vis static $name: std::sync::LazyLock<u16> = std::sync::LazyLock::new(|| $crate::LANG.id_for_node_kind($kind, true));
        }
    );
    kind_id!(pub CLASS, "class_specifier");
    kind_id!(pub NS, "namespace_definition");
    kind_id!(pub NESTED_NS, "nested_namespace_specifier");
    kind_id!(pub QUALIFIED_ID, "qualified_identifier");
    kind_id!(pub NS_ID, "namespace_identifier");
    kind_id!(pub TYPE_ID, "type_identifier");
}

pub mod f {
    macro_rules! field_id (
        ($vis:vis $name:ident, $field:literal) => {
            $vis static $name: std::sync::LazyLock<u16> = std::sync::LazyLock::new(|| $crate::LANG.field_id_for_name($field).unwrap().into());
        }
    );
    field_id!(pub SCOPE, "scope");
    field_id!(pub NAME, "name");
}

structured_query! {
    name: pub ClassDef,
    fields: {
        name: _,
        -
        fn_decl: FnDecl,
        fn_ret: FnRetType,
    },
    query: r#"
        (class_specifier
            name: (type_identifier) @name
            body: (field_declaration_list
                ((access_specifier) @variable.builtin (#eq? @variable.builtin "public"))
                .
                [(comment) (declaration) (field_declaration)]*
                .
                (field_declaration
                    declarator: (_) @fn_decl
                ) @fn_ret
            )
        )
    "#,
}

structured_query! {
    name: pub FnDef,
    fields: {
        ret: FnRetType,
        decl: FnDecl,
    },
    query: r#"
        (field_declaration_list
            ((access_specifier) @variable.builtin (#eq? @variable.builtin "public"))
            .
            [(comment) (declaration) (field_declaration)]*
            .
            (field_declaration
                declarator: (_) @decl
            ) @ret
        )
    "#
}

structured_query! {
    name: pub FnRetType,
    fields: {
        q: ?,
        t: _,
        r: [FnRetRef],
    },
    query: r#"
        (field_declaration
            (type_qualifier ("const") @q)?
            type: [(primitive_type) (type_identifier) (qualified_identifier) (template_type)] @t
        ) @r
    "#
}

structured_query! {
    name: pub FnRetRef,
    fields: {
        r: _,
    },
    query: r#"
        (_
            declarator: [
                (reference_declarator
                    [("&") ("&&")] @r
                    [(function_declarator)]
                )
                (pointer_declarator
                    ("*") @r
                    declarator: [(function_declarator) (pointer_declarator)]
                )
            ]
        )
    "#,
}

structured_query! {
    name: pub FnDecl,
    fields: {
        name: _,
        args: [FnArg],
        immut: ?,
        noexcept: ?,
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
        typ: FnArgType,
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
    name: pub FnArgType,
    fields: {
        q: ?,
        t: _,
        r: [FnArgRef],
    },
    query: r#"
        (_
            (type_qualifier ("const"))? @q
            type: [(primitive_type) (type_identifier) (qualified_identifier) (template_type)] @t
        ) @r
    "#,
}

structured_query! {
    name: pub FnArgRef,
    fields: {
        r: _,
    },
    query: r#"
        (_
            declarator: [
                (reference_declarator
                    [("&") ("&&")] @r
                    (identifier)
                )
                (pointer_declarator
                    ("*") @r
                    declarator: [(pointer_declarator) (identifier)]
                )
            ]
        )
    "#,
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Context;

    #[test]
    fn arg() {
        let code =
            "const Session& my_method(int simple_arg, const std::unique_ptr<Class>& complex_arg);";
        let expected = [
            FnArg::Literal {
                typ: FnArgType::Literal {
                    q: None,
                    t: "int",
                    r: vec![],
                },
                name: "simple_arg",
            },
            FnArg::Literal {
                typ: FnArgType::Literal {
                    q: Some("const"),
                    t: "std::unique_ptr<Class>",
                    r: vec![FnArgRef::Literal { r: "&" }],
                },
                name: "complex_arg",
            },
        ];

        let ctx = Context::from_buffer(code).expect("Parse error");
        let root = ctx.root();
        println!("{root}");
        crate::test::collection_eq(FnArg::Match::from_node(root), expected);
    }

    #[test]
    fn simple() {
        let code = "int x = 0;";
        let expected = [FnArgType::Literal {
            q: None,
            t: "int",
            r: vec![],
        }];

        let ctx = Context::from_buffer(code).expect("Failed to parse snippet");
        let root = ctx.root();
        println!("{root}");
        crate::test::collection_eq(FnArgType::Match::from_node(root), expected);
    }

    #[test]
    fn ref1() {
        let code = "Session& x = y;";
        let expected = [FnArgType::Literal {
            q: None,
            t: "Session",
            r: vec![FnArgRef::Literal { r: "&" }],
        }];

        let ctx = Context::from_buffer(code).expect("Failed to parse snippet");
        let root = ctx.root();
        println!("{root}");
        crate::test::collection_eq(FnArgType::Match::from_node(root), expected);
    }

    #[test]
    fn ref2() {
        let code = "Session&& x = y;";
        let expected = [FnArgType::Literal {
            q: None,
            t: "Session",
            r: vec![FnArgRef::Literal { r: "&&" }],
        }];

        let ctx = Context::from_buffer(code).expect("Failed to parse snippet");
        let root = ctx.root();
        println!("{root}");
        crate::test::collection_eq(FnArgType::Match::from_node(root), expected);
    }

    #[test]
    fn ptr1() {
        let code = "Session* x = y;";
        let expected = [FnArgType::Literal {
            q: None,
            t: "Session",
            r: vec![FnArgRef::Literal { r: "*" }],
        }];

        let ctx = Context::from_buffer(code).expect("Failed to parse snippet");
        let root = ctx.root();
        println!("{root}");
        crate::test::collection_eq(FnArgType::Match::from_node(root), expected);
    }

    #[test]
    fn ptr2() {
        let code = "Session** x = y;";
        let expected = [FnArgType::Literal {
            q: None,
            t: "Session",
            r: vec![FnArgRef::Literal { r: "*" }, FnArgRef::Literal { r: "*" }],
        }];

        let ctx = Context::from_buffer(code).expect("Failed to parse snippet");
        let root = ctx.root();
        println!("{root}");
        crate::test::collection_eq(FnArgType::Match::from_node(root), expected);
    }

    #[test]
    fn ptr3() {
        let code = "Session*** x = y;";
        let expected = [FnArgType::Literal {
            q: None,
            t: "Session",
            r: vec![
                FnArgRef::Literal { r: "*" },
                FnArgRef::Literal { r: "*" },
                FnArgRef::Literal { r: "*" },
            ],
        }];

        let ctx = Context::from_buffer(code).expect("Failed to parse snippet");
        let root = ctx.root();
        println!("{root}");
        crate::test::collection_eq(FnArgType::Match::from_node(root), expected);
    }

    #[test]
    fn class() {
        println!("{}", crate::test::input::CTX.root());
        crate::test::collection_eq(
            ClassDef::Match::from_node(crate::test::input::CTX.root()),
            vec![
                ClassDef::Literal {
                    name: "MyClass",
                    methods: vec![
                        crate::test::expected::PARSED.get("config").unwrap().clone(),
                        crate::test::expected::PARSED.get("set_config").unwrap().clone(),
                    ]
                },
                ClassDef::Literal {
                    name: "SubClass",
                    methods: vec![
                        crate::test::expected::PARSED.get("do_something").unwrap().clone(),
                    ]
                }
            ]
        );
    }
}
