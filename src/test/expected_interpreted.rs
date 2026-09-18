use crate::{ApiFnArg, ApiFnDef, ApiType, TypeName, TypeRelation};
use std::{collections::HashMap, sync::LazyLock};

pub static INTERPRETED: LazyLock<HashMap<&'static str, ApiFnDef>> = LazyLock::new(|| {
    HashMap::from_iter([
        (
            "do_something",
            ApiFnDef {
                name: "do_something".into(),
                fqns: TypeName::from_iter(["x", "y", "MyClass", "SubClass"]),
                ret: ApiType {
                    typ: "bool".into(),
                    relation: TypeRelation::Value,
                    is_const: false,
                },
                args: vec![ApiFnArg {
                    typ: ApiType {
                        typ: "bool".into(),
                        is_const: false,
                        relation: TypeRelation::Value,
                    },
                    name: "flag".into(),
                    is_out: false,
                }],
                noexcept: false,
                is_const: false,
            },
        ),
        (
            "config",
            ApiFnDef {
                name: "config".into(),
                fqns: TypeName::from_iter(["x", "y", "MyClass"]),
                ret: ApiType {
                    typ: "z::Config".into(),
                    relation: TypeRelation::Ref,
                    is_const: true,
                },
                args: vec![],
                noexcept: true,
                is_const: true,
            },
        ),
        (
            "set_config",
            ApiFnDef {
                name: "set_config".into(),
                fqns: TypeName::from_iter(["x", "y", "MyClass"]),
                ret: ApiType::void(),
                args: vec![ApiFnArg {
                    name: "config".into(),
                    typ: ApiType {
                        typ: "z::Config".into(),
                        is_const: true,
                        relation: TypeRelation::Ref,
                    },
                    is_out: false,
                }],
                is_const: false,
                noexcept: false,
            },
        ),
    ])
});
