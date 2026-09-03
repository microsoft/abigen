use std::fmt::Display;

use crate::{ApiFnArg, ApiFnDef, ApiTyperef, RefType};

pub struct FnDefHeader<'a>(&'a ApiFnDef);
impl<'a> From<&'a ApiFnDef> for FnDefHeader<'a> {
    fn from(value: &'a ApiFnDef) -> Self {
        Self(value)
    }
}
impl Display for FnDefHeader<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "extern \"C\" {ret} {name}(",
            ret = TyperefHeader::from(&self.0.ret),
            name = self.0.name,
        )?;

        if self.0.args.len() > 0 {
            write!(f, "{}", FnArgHeader::from(&self.0.args[0]))?;
        }
        for a in &self.0.args[1..] {
            write!(f, ", {}", FnArgHeader::from(a))?;
        }

        write!(f, ")")?;

        if self.0.immut {
            write!(f, " const")?;
        }
        if self.0.noexcept {
            write!(f, " noexcept")?;
        }

        write!(f, ";")?;
        Ok(())
    }
}

pub struct FnArgHeader<'a>(&'a ApiFnArg);
impl<'a> From<&'a ApiFnArg> for FnArgHeader<'a> {
    fn from(value: &'a ApiFnArg) -> Self {
        Self(value)
    }
}
impl Display for FnArgHeader<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{typ} {name}",
            typ = TyperefHeader::from(&self.0.typ),
            name = self.0.name,
        )?;
        Ok(())
    }
}

pub struct TyperefHeader<'a>(&'a ApiTyperef);
impl<'a> From<&'a ApiTyperef> for TyperefHeader<'a> {
    fn from(value: &'a ApiTyperef) -> Self {
        Self(value)
    }
}
impl Display for TyperefHeader<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.is_const {
            write!(f, "const ")?;
        }

        write!(f, "{name}", name = &self.0.typ)?;

        match self.0.relation {
            RefType::OwnedReference => {
                write!(f, "&& ")?;
            },
            RefType::Reference => {
                write!(f, "& ")?;
            },
            RefType::Value => {
                write!(f, " ")?;
            },
            RefType::Pointer { depth, is_const } => {
                for _ in 0..depth {
                    write!(f, "*")?;
                }
                if is_const {
                    write!(f, " const")?;
                }
            },
        }

        Ok(())
    }
}