use crate::query::{FnArg, FnDef, Typeref};
use std::str::Utf8Error;

#[derive(Debug, Clone)]
pub struct ApiFnDef {
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

        Ok(Self {
            ret: value.ret.try_into()?,
            name: value.decl.name.as_str()?.to_string(),
            args,
            noexcept: value.decl.noexcept.is_some(),
            immut: value.decl.immut.is_some(),
        })
    }
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefType {
    Pointer { depth: usize, is_const: bool },
    Value,
    Reference,
    OwnedReference,
}

impl From<Typeref::Match<'_>> for RefType {
    fn from(value: Typeref::Match<'_>) -> Self {
        if value.r.len() == 0 {
            RefType::Value
        } else if let Ok("&") = value.r[0].r.as_str() {
            RefType::Reference
        } else if let Ok("&&") = value.r[0].r.as_str() {
            RefType::OwnedReference
        } else {
            RefType::Pointer {
                depth: value.r.len(),
                is_const: value.r.iter().any(|r| r.q.is_some()),
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ApiTyperef {
    pub typ: String,
    pub is_const: bool,
    pub relation: RefType,
}

impl TryFrom<Typeref::Match<'_>> for ApiTyperef {
    type Error = Utf8Error;
    fn try_from(value: Typeref::Match<'_>) -> Result<Self, Utf8Error> {
        let is_const = if let Some(s) = value.q
            && s.as_str()? == "const"
        {
            true
        } else {
            false
        };

        Ok(Self {
            is_const,
            typ: value.t.as_str()?.to_string(),
            relation: RefType::from(value),
        })
    }
}
