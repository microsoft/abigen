use crate::{
    AbiConfig,
    query::{FnArg, FnArgType, FnDef, FnRetType},
    format::Delimited,
};
use std::{borrow::Cow, hash::Hash};
use std::fmt::{self, Display};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ApiFnDef {
    pub ret: ApiTyperef,
    pub name: String,
    pub fqns: Vec<String>,
    pub args: Vec<ApiFnArg>,
    pub noexcept: bool,
    pub immut: bool,
}

impl From<&FnDef::Match<'_>> for ApiFnDef {
    fn from(value: &FnDef::Match) -> Self {
        Self {
            ret: ApiTyperef::from(&value.ret),
            name: value.decl.name.as_str().to_string(),
            fqns: value.class.iter().map(|c| c.as_str().to_string()).collect(),
            args: value.decl.args.iter().map(|a| a.into()).collect::<Vec<_>>(),
            noexcept: value.decl.noexcept.is_some(),
            immut: value.decl.immut.is_some(),
        }
    }
}

impl ApiFnDef {
    pub fn header(&self, config: &AbiConfig) -> impl Display {
        fmt::from_fn(|f| {
            write!(f, "{};", self.proto(config))
        })
    }

    pub fn code(&self, config: &AbiConfig) -> impl Display {
        fmt::from_fn(|f| {
            write!(f, "{proto}\n{body}",
                proto = self.proto(config),
                body = self.body(config),
            )
        })
    }

    pub fn pointer(&self, _config: &AbiConfig) -> impl Display {
        todo!()
    }

    fn name_c(&self) -> impl Display {
        self.fqns.iter()
            .map(|p| p.to_lowercase())
            .chain([self.name.to_lowercase()].into_iter())
            .collect::<Delimited<'_'>>()
    }

    /// The name of the C type that opaquely stands in for the C++ type
    fn inst_handle_t(&self) -> impl Display {
        self.fqns.iter().collect::<Delimited<'_'>>()
    }

    fn args_c<'a>(&'a self) -> impl IntoIterator<Item=Cow<'a, ApiFnArg>> {
        let mut args: Vec<_> = self.args.iter().map(|a| Cow::Borrowed(a)).collect();

        let self_arg = ApiFnArg::self_arg(self.immut, self.inst_handle_t().to_string());
        args.insert(0, Cow::Owned(self_arg));

        if let Some(o) = ApiFnArg::out_arg(&self.ret) {
            args.push(Cow::Owned(o));
        }

        args
    }

    fn proto(&self, config: &AbiConfig) -> impl Display {
        fmt::from_fn(|f| {
            write!(f, "{ret} {name}({args})",
                ret = config.exception.value_type,
                name = self.name_c(),
                args = self.args_c().into_iter().map(|a| a.decl().to_string()).collect::<Delimited<','>>(),
            )
        })
    }

    fn body(&self, config: &AbiConfig) -> impl Display {
        fmt::from_fn(|f| {
            writeln!(f, "{{")?;

            for a in self.args_c() {
                a.nullcheck(config).fmt(f)?;
            }

            self.body_try_exec(config).fmt(f)?;

            writeln!(f, "return {};", config.exception.success_value)?;
            writeln!(f, "}}")
        })
    }

    fn body_try_exec(&self, config: &AbiConfig) -> impl Display {
        fmt::from_fn(|f| {
            if !self.ret.is_void() {
                writeln!(f, "*out = nullptr;")?;
            }

            writeln!(f, "try {{ {} }}", self.body_exec(config))?;

            for (catch, ret) in config.exception.custom_values.iter() {
                writeln!(f, "catch ({catch}) {{ return {ret}; }}")?;
            }

            writeln!(f, "catch (...) {{ return {}; }}", config.exception.generic_value)
        })
    }

    fn body_exec(&self, _config: &AbiConfig) -> impl Display {
        fmt::from_fn(|f| {
            let args = self.args.iter()
                .map(|a| a.deref())
                .collect::<Delimited<','>>();

            if let Some(o) = ApiFnArg::out_arg(&self.ret) {
                writeln!(f, "{out} = self->{name}({args});",
                    out = o.deref(),
                    name = self.name,
                )
            } else {
                writeln!(f, "self->{name}({args});",
                    name = self.name,
                )
            }
        })
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
    fn self_arg(is_const: bool, t: impl Into<String>) -> Self {
        Self {
            typ: ApiTyperef {
                is_const,
                typ: t.into(),
                relation: RefType::Reference,
            },
            name: "self".to_string(),
        }
    }

    fn out_arg(t: &ApiTyperef) -> Option<Self> {
        if !t.is_void() {
            Some(Self {
                typ: ApiTyperef {
                    is_const: false,
                    typ: t.typ.clone(),
                    relation: if let RefType::Pointer(d) = t.relation {
                        RefType::Pointer(d+1)
                    } else {
                        RefType::OutReference
                    }
                },
                name: "out".to_string(),
            })
        } else {
            None
        }
    }
    
    fn nullcheck(&self, config: &AbiConfig) -> impl Display {
        fmt::from_fn(|f| {
            let depth = match self.typ.relation {
                RefType::OutReference => 1,
                RefType::Reference => 1,
                RefType::Value => 0,
                RefType::Pointer(d) => d,
            };

            for i in 0..depth {
                write!(f, "if (")?;
                for _ in 0..i {
                    write!(f, "*")?;
                }
                writeln!(f, "{name} == nullptr) {{ return {nullref_exception}; }}",
                    name = self.name,
                    nullref_exception = config.exception.nullref_value,
                )?;
            }

            Ok(())
        })
    }

    fn decl(&self) -> impl Display {
        fmt::from_fn(|f| {
            write!(f, "{typ} {name}",
                typ = self.typ,
                name = self.name,
            )
        })
    }

    fn deref(&self) -> impl Display {
        fmt::from_fn(|f| {
            match self.typ.relation {
                RefType::OutReference | RefType::Reference => write!(f, "*{}", self.name),
                RefType::Value | RefType::Pointer(_) => write!(f, "{}", self.name),
            }
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RefType {
    #[default]
    Value,
    Pointer(usize),
    Reference,
    OutReference,
}

impl From<&FnArgType::Match<'_>> for RefType {
    fn from(value: &FnArgType::Match<'_>) -> Self {
        if value.r.len() == 0 {
            RefType::Value
        } else if value.r[0].r.as_str() == "&" {
            RefType::Reference
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
            RefType::OutReference
        } else {
            RefType::Pointer(value.r.len())
        }
    }
}

impl Display for RefType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutReference | Self::Pointer(2) => write!(f, "**"),
            Self::Reference | Self::Pointer(1) => write!(f, "*"),
            Self::Value | Self::Pointer(0) => Ok(()),
            Self::Pointer(d) => {
                for _ in 0..*d {
                    write!(f, "*")?;
                }
                Ok(())
            }
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

    pub fn is_void(&self) -> bool {
        self.typ == "void"
    }
}

impl Display for ApiTyperef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{const} {typ} {rel}",
            const = if self.is_const { "const" } else { "" },
            typ = self.typ,
            rel = self.relation,
        )
    }
}