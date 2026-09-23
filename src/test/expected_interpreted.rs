use crate::{ApiClassDef, ApiFnArg, ApiFnDef, ApiFnRole, ApiType, TypeName, TypeRelation};
use std::{
    collections::{HashMap, HashSet},
    sync::LazyLock,
};

/// Maps the fully qualified class name to its api structure
pub static INTERPRETED: LazyLock<HashMap<TypeName, ApiClassDef>> = LazyLock::new(|| {
    HashMap::from_iter([
        (
            "x::y::MyClass::SubClass".into(),
            ApiClassDef {
                name: "x::y::MyClass::SubClass".into(),
                methods: HashSet::from_iter([ApiFnDef {
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
                    noexcept: false,
                    is_const: false,
                }]),
            },
        ),
        (
            "x::y::MyClass".into(),
            ApiClassDef {
                name: "x::y::MyClass".into(),
                methods: HashSet::from_iter([
                    ApiFnDef {
                        role: ApiFnRole::Constructor,
                        args: vec![],
                        noexcept: true,
                        is_const: false,
                    },
                    ApiFnDef {
                        role: ApiFnRole::Destructor,
                        args: vec![],
                        noexcept: true,
                        is_const: false,
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
                        args: vec![],
                        noexcept: true,
                        is_const: true,
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
                        is_const: false,
                        noexcept: false,
                    },
                ]),
            },
        ),
    ])
});
