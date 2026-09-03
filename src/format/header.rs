use crate::{ApiFnArg, ApiFnDef, ApiTyperef, RefType};
use std::borrow::Cow;
use std::fmt::Display;

impl ApiFnDef {
    pub fn header(&self) -> FnDefHeader<'_> {
        FnDefHeader::from(self)
    }

    pub fn into_header(self) -> FnDefHeader<'static> {
        FnDefHeader::from(self)
    }
}

pub struct FnDefHeader<'a>(Cow<'a, ApiFnDef>);
impl<'a> From<&'a ApiFnDef> for FnDefHeader<'a> {
    fn from(value: &'a ApiFnDef) -> Self {
        Self(Cow::Borrowed(value))
    }
}
impl<'a> From<ApiFnDef> for FnDefHeader<'a> {
    fn from(value: ApiFnDef) -> Self {
        Self(Cow::Owned(value))
    }
}
impl Display for FnDefHeader<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "extern \"C\" {ret} {name}(",
            ret = TyperefHeader::from(&self.0.ret),
            name = self.0.name,
        )?;

        for (i, arg) in self.0.args.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", FnArgHeader::from(arg))?;
        }

        write!(f, ");")?;
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
        write!(
            f,
            "{typ} {name}",
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
                write!(f, "&&")?;
            }
            RefType::Reference => {
                write!(f, "&")?;
            }
            RefType::Value => {}
            RefType::Pointer(depth) => {
                for _ in 0..depth {
                    write!(f, "*")?;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{Context, query::FnDef};

    #[test]
    fn print() {
        let ctx = Context::from_buffer(crate::test_util::CODE).expect("Failed to parse snippet");
        let root = ctx.root();
        let defs: Vec<_> = FnDef::Match::from_node(root)
            .into_iter()
            .map(|q| {
                ApiFnDef::try_from(q)
                    .expect("Failed to convert")
                    .to_c(&crate::test_util::CONFIG)
                    .into_header()
                    .to_string()
            })
            .collect();
        crate::test_util::collection_eq(
            defs,
            [
                "extern \"C\" Result myclass_config(const MyClass* this, Config** out);",
                "extern \"C\" Result myclass_set_config(MyClass* this, const Config* config);",
            ],
        );
    }
}
