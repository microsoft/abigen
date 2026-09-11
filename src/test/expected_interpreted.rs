use std::{collections::HashMap, sync::LazyLock};
use crate::{ApiFnDef, ApiFnArg, ApiTyperef, RefType};

pub static INTERPRETED_EXPECTED: LazyLock<HashMap<&'static str, ApiFnDef>> = LazyLock::new(|| {
    HashMap::from_iter([
        ("config", ApiFnDef {
            class: Some("MyClass".to_string()),
            name: "config".to_string(),
            ret: ApiTyperef {
                typ: "Config".to_string(),
                relation: RefType::Reference,
                is_const: true,
            },
            args: vec![],
            noexcept: true,
            immut: true,
        }),
        ("set_config", ApiFnDef {
            class: Some("MyClass".to_string()),
            name: "set_config".to_string(),
            ret: ApiTyperef::void(),
            args: vec![
                ApiFnArg {
                    name: "config".to_string(),
                    typ: ApiTyperef {
                        typ: "Config".to_string(),
                        is_const: true,
                        relation: RefType::Reference,
                    },
                },
            ],
            immut: false,
            noexcept: false,
        }),
    ])
});