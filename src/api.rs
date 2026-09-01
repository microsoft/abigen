use std::{path::Path, str::Utf8Error};

use crate::{
    Context, ContextError,
    query::{FnArg, FnDef, Typeref},
};

#[derive(Debug, Clone, Default)]
pub struct ApiHeader {
    pub functions: Vec<ApiFnDef>,
}

impl ApiHeader {
    pub fn from_file(file: &Path) -> Result<Self, ContextError> {
        let mut s = Self::default();

        let ctx = Context::from_file(file)?;
        for m in FnDef::Match::from_node(ctx.root()) {
            s.functions.push(m.try_into()?);
        }

        Ok(s)
    }
}

#[derive(Debug, Clone)]
pub struct ApiFnDef {
    pub ret: ApiTyperef,
    pub name: String,
    pub args: Vec<ApiFnArg>,
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

#[derive(Debug, Clone)]
pub struct ApiTyperef {
    pub name: String,
    pub is_const: bool,
    pub is_ref: bool,
}

impl TryFrom<Typeref::Match<'_>> for ApiTyperef {
    type Error = Utf8Error;
    fn try_from(value: Typeref::Match<'_>) -> Result<Self, Utf8Error> {
        Ok(Self {
            is_const: matches!(value.q.and_then(|s| s.as_str().ok()), Some("const")),
            name: value.t.as_str()?.to_string(),
            is_ref: value.r.len() > 0,
        })
    }
}
