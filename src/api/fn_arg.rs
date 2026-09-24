use crate::{AbiConfig, ApiClassDef, ApiFnDef, ApiFnRole, ApiType, TypeRelation, TypeContext, format, query::FnArg};
use std::{borrow::Cow, fmt};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ApiFnArg {
    pub typ: ApiType,
    pub name: String,
    pub is_out: bool,
}

impl From<&FnArg::Match<'_>> for ApiFnArg {
    fn from(value: &FnArg::Match) -> Self {
        Self {
            typ: ApiType::from(value),
            name: value.name.as_str().to_string(),
            is_out: false,
        }
    }
}

impl<'a> Into<Cow<'a, ApiFnArg>> for ApiFnArg {
    fn into(self) -> Cow<'a, ApiFnArg> {
        Cow::Owned(self)
    }
}
impl<'a> Into<Cow<'a, ApiFnArg>> for &'a ApiFnArg {
    fn into(self) -> Cow<'a, ApiFnArg> {
        Cow::Borrowed(self)
    }
}

impl ApiFnArg {
    pub(crate) fn self_arg(class_def: &ApiClassDef, method_def: &ApiFnDef) -> Option<Self> {
        match &method_def.role {
            ApiFnRole::Constructor => None,
            ApiFnRole::Destructor => Some(Self {
                typ: ApiType {
                    is_const: false,
                    typ: class_def.name.clone(),
                    relation: TypeRelation::Owned,
                },
                name: "self_ptr".into(),
                is_out: false,
            }),
            ApiFnRole::Method { .. } => Some(Self {
                typ: ApiType {
                    is_const: method_def.is_const,
                    typ: class_def.name.clone(),
                    relation: TypeRelation::Ref,
                },
                name: "self_ptr".to_string(),
                is_out: false,
            }),
        }
    }

    pub(crate) fn out_arg(class_def: &ApiClassDef, method_def: &ApiFnDef) -> Option<Self> {
        match &method_def.role {
            ApiFnRole::Constructor if method_def.noexcept => None,
            ApiFnRole::Constructor => Some(Self {
                typ: ApiType {
                    is_const: false,
                    typ: class_def.name.clone(),
                    relation: TypeRelation::Owned,
                },
                name: "out".into(),
                is_out: true,
            }),
            ApiFnRole::Destructor => None,
            ApiFnRole::Method { ret, .. } if ret.is_void() => None,
            ApiFnRole::Method { ret, .. } => Some(Self {
                typ: ApiType {
                    is_const: false,
                    typ: ret.typ.clone(),
                    relation: ret.relation,
                },
                name: "out".into(),
                is_out: method_def.needs_trycatch(),
            }),
        }
    }

    pub(crate) fn nullcheck(&self, f: &mut fmt::Formatter<'_>, config: &AbiConfig) -> fmt::Result {
        let ctx = if self.is_out {
            TypeContext::OutArg
        } else {
            TypeContext::Outbound
        };
        let depth = self.typ.relation.nullcheck_depth(ctx);
        for i in 0..depth {
            write!(
                f,
                "if ({deref}{name} == nullptr) {{ return {nullref_exception}; }}",
                deref = format::fmt_wrap!(format::repeat, "*", i),
                name = self.name,
                nullref_exception = config.exception.nullref_value,
            )?;
            if i < depth - 1 {
                writeln!(f)?;
            }
        }
        Ok(())
    }

    pub(crate) fn decl(
        &self,
        f: &mut fmt::Formatter<'_>,
        config: &AbiConfig,
        is_external: bool,
    ) -> fmt::Result {
        let ctx = if self.is_out {
            TypeContext::OutArg
        } else if is_external {
            TypeContext::Outbound
        } else {
            TypeContext::Inbound
        };
        self.typ.decl(f, config, ctx)?;
        write!(f, " {}", self.name)
    }

    /// Write
    pub(crate) fn deref(
        &self,
        f: &mut fmt::Formatter<'_>,
        config: &AbiConfig,
        is_external: bool,
    ) -> fmt::Result {
        let ctx = if self.is_out {
            TypeContext::OutArg
        } else if is_external {
            TypeContext::Outbound
        } else {
            TypeContext::Inbound
        };
        self.typ.deref_pre_name(f, config, ctx)?;
        write!(f, "{}", self.name)?;
        self.typ.deref_post_name(f, config, ctx)
    }
}
