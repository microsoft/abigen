use crate::query::*;
use std::{collections::HashMap, sync::LazyLock};

pub static PARSED: LazyLock<HashMap<&'static str, FnDef::Literal>> = LazyLock::new(|| {
    HashMap::from_iter([
        (
            "do_something",
            FnDef::Literal {
                ret: FnRetType::Literal {
                    q: None,
                    t: "bool",
                    r: vec![],
                },
                decl: FnDecl::Literal {
                    name: "do_something",
                    args: vec![FnArg::Literal {
                        name: "flag",
                        typ: FnArgType::Literal { q: None, t: "bool", r: vec![] },
                    }],
                    immut: None,
                    noexcept: None,
                },
            },
        ),
        (
            "config",
            FnDef::Literal {
                ret: FnRetType::Literal {
                    q: Some("const"),
                    t: "z::Config",
                    r: vec![FnRetRef::Literal { r: "&" }],
                },
                decl: FnDecl::Literal {
                    name: "config",
                    args: vec![],
                    immut: Some("const"),
                    noexcept: Some("noexcept"),
                },
            },
        ),
        (
            "set_config",
            FnDef::Literal {
                ret: FnRetType::Literal {
                    q: None,
                    t: "void",
                    r: vec![],
                },
                decl: FnDecl::Literal {
                    name: "set_config",
                    args: vec![FnArg::Literal {
                        name: "config",
                        typ: FnArgType::Literal {
                            q: Some("const"),
                            t: "z::Config",
                            r: vec![FnArgRef::Literal { r: "&" }],
                        },
                    }],
                    immut: None,
                    noexcept: None,
                },
            },
        ),
    ])
});
