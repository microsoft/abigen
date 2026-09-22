use crate::query::*;
use std::{collections::HashMap, sync::LazyLock};

/// Maps the unqualified class name to its query structure
pub static PARSED: LazyLock<HashMap<&'static str, ClassDef::Literal>> = LazyLock::new(|| {
    HashMap::from_iter([
        (
            "SubClass",
            ClassDef::Literal {
                name: "SubClass",
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
                        immut: None,
                        noexcept: None,
                    },
                }],
            },
        ),
        (
            "MyClass",
            ClassDef::Literal {
                name: "MyClass",
                methods: vec![
                    FnDef::Literal {
                        ret_qual: Some("const"),
                        ret_typ: "z::Config",
                        ret_refs: vec![FnRetRef::Literal { r: "&" }],
                        decl: FnDecl::Literal {
                            name: "config",
                            args: vec![],
                            immut: Some("const"),
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
                            immut: None,
                            noexcept: None,
                        },
                    },
                ],
            },
        ),
    ])
});
