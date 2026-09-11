use crate::{
    AbiConfig,
    query::{FnArg, FnArgType, FnDef, FnRetType},
};
use std::hash::Hash;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ApiFnDef {
    pub class: Option<String>,
    pub ret: ApiTyperef,
    pub name: String,
    pub args: Vec<ApiFnArg>,
    pub noexcept: bool,
    pub immut: bool,
}

impl From<&FnDef::Match<'_>> for ApiFnDef {
    fn from(value: &FnDef::Match) -> Self {
        let args_r: Vec<_> = value.decl.args.iter().map(|a| a.into()).collect();
        let mut args = vec![];
        for r in args_r {
            args.push(r);
        }

        let class = Some(value.class.as_ref().unwrap().as_str().to_string());
        Self {
            class,
            ret: ApiTyperef::from(&value.ret),
            name: value.decl.name.as_str().to_string(),
            args,
            noexcept: value.decl.noexcept.is_some(),
            immut: value.decl.immut.is_some(),
        }
    }
}

impl ApiFnDef {
    pub fn to_c(&self, config: &AbiConfig) -> Self {
        let mut c = self.clone();
        let class = c.class.clone().unwrap(); // cannot fail, enforced by query

        c.name = format!(
            "{class}_{name}",
            class = class.to_lowercase(),
            name = c.name
        );

        c.args.insert(
            0,
            ApiFnArg {
                typ: ApiTyperef {
                    typ: class.clone(),
                    is_const: c.immut,
                    relation: RefType::Pointer(1),
                },
                name: "self".to_string(),
            },
        );

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
            typ: config.exception.value_type.clone(),
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

impl From<&FnArg::Match<'_>> for ApiFnArg {
    fn from(value: &FnArg::Match) -> Self {
        Self {
            typ: ApiTyperef::from(&value.typ),
            name: value.name.as_str().to_string(),
        }
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

impl From<&FnArgType::Match<'_>> for RefType {
    fn from(value: &FnArgType::Match<'_>) -> Self {
        if value.r.len() == 0 {
            RefType::Value
        } else if value.r[0].r.as_str() == "&" {
            RefType::Reference
        } else if value.r[0].r.as_str() == "&&" {
            RefType::OwnedReference
        } else {
            RefType::Pointer(value.r.len())
        }
    }
}

impl From<&FnRetType::Match<'_>> for RefType {
    fn from(value: &FnRetType::Match<'_>) -> Self {
        if value.r.len() == 0 {
            RefType::Value
        } else if value.r[0].r.as_str() == "&" {
            RefType::Reference
        } else if value.r[0].r.as_str() == "&&" {
            RefType::OwnedReference
        } else {
            RefType::Pointer(value.r.len())
        }
    }
}

impl RefType {
    pub fn to_c(&self, _config: &AbiConfig) -> Self {
        match self {
            Self::OwnedReference => Self::Pointer(2),
            Self::Reference => Self::Pointer(1),
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

impl From<&FnArgType::Match<'_>> for ApiTyperef {
    fn from(value: &FnArgType::Match<'_>) -> Self {
        Self {
            is_const: value.q.is_some(),
            typ: value.t.as_str().to_string(),
            relation: RefType::from(value),
        }
    }
}

impl From<&FnRetType::Match<'_>> for ApiTyperef {
    fn from(value: &FnRetType::Match<'_>) -> Self {
        Self {
            is_const: value.q.is_some(),
            typ: value.t.as_str().to_string(),
            relation: RefType::from(value),
        }
    }
}

impl ApiTyperef {
    pub fn void() -> Self {
        Self {
            typ: "void".to_string(),
            is_const: false,
            relation: RefType::Value,
        }
    }

    pub fn to_c(&self, config: &AbiConfig) -> Self {
        let mut c = self.clone();
        c.relation = c.relation.to_c(config);
        c
    }
}
