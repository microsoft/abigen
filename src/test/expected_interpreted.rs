use crate::{ApiClassDef, ApiFnArg, ApiFnDef, ApiFnRole, ApiType, TypeName, TypeRelation};
use std::{collections::HashMap, sync::LazyLock};

/// Maps the fully qualified class name to its api structure
pub static INTERPRETED: LazyLock<HashMap<TypeName, ApiClassDef>> = LazyLock::new(|| {
    [
        ApiClassDef::new(
            "x::y::MyClass::SubClass",
            [ApiFnDef {
                role: ApiFnRole::Method {
                    name: "do_something".into(),
                    ret: ApiType {
                        typ: "bool".into(),
                        relation: TypeRelation::Value,
                        is_const: false,
                    },
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
                ..Default::default()
            }],
        ),
        ApiClassDef::new(
            "x::y::MyClass",
            [
                // default constructor
                ApiFnDef {
                    role: ApiFnRole::Constructor,
                    noexcept: true,
                    ..Default::default()
                },
                // move constructor
                ApiFnDef {
                    role: ApiFnRole::Constructor,
                    args: vec![ApiFnArg {
                        name: "other".into(),
                        typ: ApiType {
                            typ: "x::y::MyClass".into(),
                            is_const: false,
                            relation: TypeRelation::Owned,
                        },
                        is_out: false,
                    }],
                    ..Default::default()
                },
                ApiFnDef {
                    role: ApiFnRole::Constructor,
                    args: vec![ApiFnArg {
                        name: "flag".into(),
                        typ: ApiType {
                            typ: "bool".into(),
                            is_const: false,
                            relation: TypeRelation::Value,
                        },
                        is_out: false,
                    }],
                    ..Default::default()
                },
                ApiFnDef {
                    role: ApiFnRole::Constructor,
                    noexcept: true,
                    ..Default::default()
                },
                ApiFnDef {
                    role: ApiFnRole::Destructor,
                    noexcept: true,
                    ..Default::default()
                },
                ApiFnDef {
                    role: ApiFnRole::Method {
                        name: "config".into(),
                        ret: ApiType {
                            typ: "z::Config".into(),
                            relation: TypeRelation::Ref,
                            is_const: true,
                        },
                    },
                    noexcept: true,
                    is_const: true,
                    ..Default::default()
                },
                ApiFnDef {
                    role: ApiFnRole::Method {
                        name: "set_config".into(),
                        ret: ApiType::void(),
                    },
                    args: vec![ApiFnArg {
                        name: "config".into(),
                        typ: ApiType {
                            typ: "z::Config".into(),
                            is_const: true,
                            relation: TypeRelation::Ref,
                        },
                        is_out: false,
                    }],
                    ..Default::default()
                },
            ],
        ),
    ]
    .into_iter()
    .map(|c| (c.name.clone(), c))
    .collect()
});
