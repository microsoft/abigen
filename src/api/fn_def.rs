use crate::{AbiConfig, query::{FnArg, FnDef, FnArgType, FnRetType}};
use std::{str::Utf8Error, hash::Hash};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ApiFnDef {
    pub class: Option<String>,
    pub ret: ApiTyperef,
    pub name: String,
    pub args: Vec<ApiFnArg>,
    pub noexcept: bool,
    pub immut: bool,
}

impl TryFrom<FnDef::Match<'_>> for ApiFnDef {
    type Error = Utf8Error;
    fn try_from(value: FnDef::Match) -> Result<Self, Utf8Error> {
        let args_r: Vec<Result<_, _>> = value.decl.args.into_iter().map(|a| a.try_into()).collect();
        let mut args = vec![];
        for r in args_r {
            args.push(r?);
        }

        let class = Some(value.class.unwrap().as_str()?.to_string());
        Ok(Self {
            class,
            ret: value.ret.try_into()?,
            name: value.decl.name.as_str()?.to_string(),
            args,
            noexcept: value.decl.noexcept.is_some(),
            immut: value.decl.immut.is_some(),
        })
    }
}

impl ApiFnDef {
    pub fn to_c(&self, config: &AbiConfig) -> Self {
        let mut c = self.clone();
        let class = c.class.clone().unwrap(); // cannot fail, enforced by query

        c.name = format!("{class}_{name}", class = class.to_lowercase(), name = c.name);

        c.args.insert(0, ApiFnArg {
            typ: ApiTyperef {
                typ: class.clone(),
                is_const: c.immut,
                relation: RefType::Pointer(1),
            },
            name: "this".to_string(),
        });

        if &c.ret.typ != "void" {
            c.args.push(ApiFnArg {
                typ: ApiTyperef {
                    typ: c.ret.typ,
                    relation: RefType::Pointer(2),
                    is_const: false,
                },
                name: "out".to_string(),
            });
        }

        c.ret = ApiTyperef {
            typ: config.exception.result_type.clone(),
            ..Default::default()
        };

        c.args = c.args.into_iter().map(|a| a.to_c(config)).collect();

        c
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ApiFnArg {
    pub typ: ApiTyperef,
    pub name: String,
}

impl TryFrom<FnArg::Match<'_>> for ApiFnArg {
    type Error = Utf8Error;
    fn try_from(value: FnArg::Match) -> Result<Self, Utf8Error> {
        Ok(Self {
            typ: value.typ.try_into()?,
            name: value.name.as_str()?.to_string(),
        })
    }
}

impl ApiFnArg {
    pub fn to_c(&self, config: &AbiConfig) -> Self {
        let mut c = self.clone();
        c.typ = c.typ.to_c(config);
        c
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RefType {
    Pointer(usize),
    #[default]
    Value,
    Reference,
    OwnedReference,
}

impl From<FnArgType::Match<'_>> for RefType {
    fn from(value: FnArgType::Match<'_>) -> Self {
        if value.r.len() == 0 {
            RefType::Value
        } else if let Ok("&") = value.r[0].r.as_str() {
            RefType::Reference
        } else if let Ok("&&") = value.r[0].r.as_str() {
            RefType::OwnedReference
        } else {
            RefType::Pointer(value.r.len())
        }
    }
}

impl From<FnRetType::Match<'_>> for RefType {
    fn from(value: FnRetType::Match<'_>) -> Self {
        if value.r.len() == 0 {
            RefType::Value
        } else if let Ok("&") = value.r[0].r.as_str() {
            RefType::Reference
        } else if let Ok("&&") = value.r[0].r.as_str() {
            RefType::OwnedReference
        } else {
            RefType::Pointer(value.r.len())
        }
    }
}

impl RefType {
    pub fn to_c(&self, _config: &AbiConfig) -> Self {
        match self {
            Self::OwnedReference => {
                Self::Pointer(2)
            },
            Self::Reference => {
                Self::Pointer(1)
            },
            t => t.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct ApiTyperef {
    pub typ: String,
    pub is_const: bool,
    pub relation: RefType,
}

impl TryFrom<FnArgType::Match<'_>> for ApiTyperef {
    type Error = Utf8Error;
    fn try_from(value: FnArgType::Match<'_>) -> Result<Self, Utf8Error> {
        Ok(Self {
            is_const: value.q.is_some(),
            typ: value.t.as_str()?.to_string(),
            relation: RefType::from(value),
        })
    }
}

impl TryFrom<FnRetType::Match<'_>> for ApiTyperef {
    type Error = Utf8Error;
    fn try_from(value: FnRetType::Match<'_>) -> Result<Self, Self::Error> {
        Ok(Self {
            is_const: value.q.is_some(),
            typ: value.t.as_str()?.to_string(),
            relation: RefType::from(value),
        })
    }
}

impl ApiTyperef {
    pub fn to_c(&self, config: &AbiConfig) -> Self {
        let mut c = self.clone();
        c.relation = c.relation.to_c(config);
        c
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Context;

    #[test]
    fn to_c() {
        let ctx = Context::from_buffer(crate::test_util::CODE).expect("Failed to parse snippet");
        let root = ctx.root();
        println!("{root}");
        
        let defs: Vec<_> = FnDef::Match::from_node(root)
            .into_iter()
            .map(|q| {
                ApiFnDef::try_from(q)
                    .expect("Failed to convert")
                    .to_c(&crate::test_util::CONFIG)
            }).collect();
        crate::test_util::collection_eq(defs, [
            ApiFnDef {
                class: Some("MyClass".to_string()),
                name: "myclass_config".to_string(),
                ret: ApiTyperef {
                    typ: crate::test_util::CONFIG.exception.result_type.clone(),
                    ..Default::default()
                },
                args: vec![
                    ApiFnArg {
                        name: "this".to_string(),
                        typ: ApiTyperef {
                            typ: "MyClass".to_string(),
                            is_const: true,
                            relation: RefType::Pointer(1),
                        }
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
            },
            ApiFnDef {
                class: Some("MyClass".to_string()),
                name: "myclass_set_config".to_string(),
                ret: ApiTyperef {
                    typ: crate::test_util::CONFIG.exception.result_type.clone(),
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
            }
        ]);

        
    }
}