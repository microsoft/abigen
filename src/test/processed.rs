use std::{collections::HashMap, sync::LazyLock};
use crate::{ApiFnDef, ApiFnArg, ApiTyperef, RefType};

pub static PROCESSED: LazyLock<HashMap<&'static str, ApiFnDef>> = LazyLock::new(|| {
    HashMap::from_iter([
        ("config", ApiFnDef {
            class: Some("MyClass".to_string()),
            name: "myclass_config".to_string(),
            ret: ApiTyperef {
                typ: crate::test::CONFIG.exception.value_type.clone(),
                ..Default::default()
            },
            args: vec![
                ApiFnArg {
                    name: "this".to_string(),
                    typ: ApiTyperef {
                        typ: "MyClass".to_string(),
                        is_const: true,
                        relation: RefType::Pointer(1),
                    },
                },
                ApiFnArg {
                    name: "out".to_string(),
                    typ: ApiTyperef {
                        typ: "Config".to_string(),
                        is_const: false,
                        relation: RefType::Pointer(2),
                    },
                },
            ],
            noexcept: true,
            immut: true,
        }),
        ("set_config", ApiFnDef {
            class: Some("MyClass".to_string()),
            name: "myclass_set_config".to_string(),
            ret: ApiTyperef {
                typ: crate::test::CONFIG.exception.value_type.clone(),
                ..Default::default()
            },
            args: vec![
                ApiFnArg {
                    name: "this".to_string(),
                    typ: ApiTyperef {
                        typ: "MyClass".to_string(),
                        is_const: false,
                        relation: RefType::Pointer(1),
                    },
                },
                ApiFnArg {
                    name: "config".to_string(),
                    typ: ApiTyperef {
                        typ: "Config".to_string(),
                        is_const: true,
                        relation: RefType::Pointer(1),
                    },
                },
            ],
            immut: false,
            noexcept: false,
        }),
    ])
});