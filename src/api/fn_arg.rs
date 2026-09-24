use crate::{AbiConfig, ApiType, TypeContext, format, query::FnArg};
use std::{borrow::Cow, fmt};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ApiFnArg {
    pub typ: ApiType,
    pub name: String,
    /// False for normal arguments and [ApiFnDef::out_arg]s as return values. True for out_args as parameters.
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
