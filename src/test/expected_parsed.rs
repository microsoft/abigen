use std::{collections::HashMap, sync::LazyLock};
use crate::query::*;

pub static PARSED_EXPECTED: LazyLock<HashMap<&'static str, FnDef::Literal>> = LazyLock::new(|| {
    HashMap::from_iter([
        ("config", FnDef::Literal {
            class: Some("MyClass"),
            ret: FnRetType::Literal {
                q: Some("const"),
                t: "Config",
                r: vec![FnRetRef::Literal { r: "&" }],
            },
            decl: FnDecl::Literal {
                name: "config",
                args: vec![],
                immut: Some("const"),
                noexcept: Some("noexcept"),
            },
        }),
        ("set_config", FnDef::Literal {
            class: Some("MyClass"),
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
                        t: "Config",
                        r: vec![FnArgRef::Literal { r: "&" }],
                    },
                }],
                immut: None,
                noexcept: None,
            },
        }),
    ])
});