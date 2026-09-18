use crate::{ApiFnArg, ApiFnDef, ApiFnRole, ApiType, TypeName, TypeRelation};
use std::{collections::HashMap, sync::LazyLock};

pub static INTERPRETED: LazyLock<HashMap<&'static str, ApiFnDef>> = LazyLock::new(|| {
    HashMap::from_iter([
        (
            "do_something",
            ApiFnDef {
                role: ApiFnRole::Method {
                    name: "do_something".into(),
                    ret: ApiType {
                        typ: "bool".into(),
                        relation: TypeRelation::Value,
                        is_const: false,
                    },
                },
                fqns: TypeName::from_iter(["x", "y", "MyClass", "SubClass"]),
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
                role: ApiFnRole::Method {
                    name: "config".into(),
                    ret: ApiType {
                        typ: "z::Config".into(),
                        relation: TypeRelation::Ref,
                        is_const: true,
                    },
                },
                fqns: TypeName::from_iter(["x", "y", "MyClass"]),
                args: vec![],
                noexcept: true,
                is_const: true,
            },
        ),
        (
            "set_config",
            ApiFnDef {
                role: ApiFnRole::Method {
                    name: "set_config".into(),
                    ret: ApiType::void(),
                },
                fqns: TypeName::from_iter(["x", "y", "MyClass"]),
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
