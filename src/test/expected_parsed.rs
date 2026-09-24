use crate::query::*;
use std::{collections::HashMap, sync::LazyLock};

/// Maps the unqualified class name to its query structure
pub static PARSED: LazyLock<HashMap<&'static str, ClassDef::Literal>> = LazyLock::new(|| [
    ClassDef::Literal {
        name: "SubClass",
        ctor: vec![],
        methods: vec![FnDef::Literal {
            ret_qual: None,
            ret_typ: "bool",
            ret_refs: vec![],
            decl: FnDecl::Literal {
                name: "do_something",
                args: vec![FnArg::Literal {
                    name: "flag",
                    typ_qual: None,
                    typ: "bool",
                    typ_refs: vec![],
                }],
                is_const: None,
                noexcept: None,
            },
        }],
    },
    ClassDef::Literal {
        name: "MyClass",
        ctor: vec![FnDecl::Literal {
            name: "MyClass",
            args: vec![],
            is_const: None,
            noexcept: Some("noexcept"),
        }],
        methods: vec![
            FnDef::Literal {
                ret_qual: Some("const"),
                ret_typ: "z::Config",
                ret_refs: vec![FnRetRef::Literal { r: "&" }],
                decl: FnDecl::Literal {
                    name: "config",
                    args: vec![],
                    is_const: Some("const"),
                    noexcept: Some("noexcept"),
                },
            },
            FnDef::Literal {
                ret_qual: None,
                ret_typ: "void",
                ret_refs: vec![],
                decl: FnDecl::Literal {
                    name: "set_config",
                    args: vec![FnArg::Literal {
                        name: "config",
                        typ_qual: Some("const"),
                        typ: "z::Config",
                        typ_refs: vec![FnArgRef::Literal { r: "&" }],
                    }],
                    is_const: None,
                    noexcept: None,
                },
            },
        ],
    },
].into_iter().map(|c| (c.name, c)).collect());
