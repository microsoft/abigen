use crate::{
    AbiConfig,
    query::{FnArg, FnArgType, FnDef, FnRetType},
    format::Delimited,
};
use std::hash::Hash;
use std::fmt::{self, Display};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ApiFnDef {
    pub ret: ApiTyperef,
    pub name_parts: Vec<String>,
    pub args: Vec<ApiFnArg>,
    pub noexcept: bool,
    pub immut: bool,
}

impl From<&FnDef::Match<'_>> for ApiFnDef {
    fn from(value: &FnDef::Match) -> Self {
        let mut name_parts = vec![value.decl.name.as_str().to_string()];
        if value.class.is_some() {
            name_parts.insert(0, value.class.as_ref().unwrap().as_str().to_string());
        }

        let args_r: Vec<_> = value.decl.args.iter().map(|a| a.into()).collect();
        let mut args = vec![];
        for r in args_r {
            args.push(r);
        }

        Self {
            ret: ApiTyperef::from(&value.ret),
            name_parts,
            args,
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

    fn name(&self) -> &str {
        self.name_parts[self.name_parts.len() - 1].as_str()
    }

    fn proto(&self, config: &AbiConfig) -> impl Display {
        fmt::from_fn(|f| {
            let mut args: Vec<_> = self.args.iter().collect();

            let self_arg = ApiFnArg::self_arg(self.immut, self.name_parts[0].as_str());
            args.insert(0, &self_arg);

            let out = ApiFnArg::out_arg(self.ret.typ.as_str());
            if self.ret.typ.as_str() != "void" {
                args.push(&out);
            }

            write!(f, "{ret} {name}({args})",
                ret = config.exception.value_type,
                name = self.name_parts.iter().map(|n| n.to_lowercase()).collect::<Delimited<'_', _>>(),
                args = args.into_iter().collect::<Delimited<',', _>>(),
            )
        })
    }

    fn body(&self, config: &AbiConfig) -> impl Display {
        fmt::from_fn(|f| {
            let nullchecks = self.args.iter()
                .map(|a| a.nullcheck(config))
                .collect::<Delimited<' ', _>>();
            let exec = fmt::from_fn(|f| {
                if self.noexcept {
                    write!(f, "{}", self.body_exec(config))
                } else {
                    write!(f, "{}", self.body_try_exec(config))
                }
            });
            writeln!(f, r#"
                {{
                    {nullchecks}
                    {exec}
                    return {success};
                }}"#,
                success = config.exception.success_value,
            )
        })
    }

    fn body_try_exec(&self, config: &AbiConfig) -> impl Display {
        fmt::from_fn(|f| {
            let custom_exceptions = config.exception.custom_values
                .iter()
                .map(|(err, ret)| {
                    fmt::from_fn(move |f| {
                        write!(f, "catch ({err}) {{ return {ret}; }}")
                    })
                })
                .collect::<Delimited<'\n', _>>();

            write!(f, r#"
                try {{ {exec} }}
                {custom_exceptions}
                catch (...) {{ return {generic_exception}; }}
            "#,
                exec = self.body_exec(config),
                generic_exception = config.exception.generic_value,
            )
        })
    }

    fn body_exec(&self, _config: &AbiConfig) -> impl Display {
        fmt::from_fn(|f| {
            if let Some("out") = self.args.last().map(|a| a.name.as_str()) {
                // no "self", no "out"
                let other_args = self.args[1..self.args.len()-1]
                    .iter()
                    .map(|a| {
                        fmt::from_fn(|f| {
                            write!(f, "{}", a.name)
                        })
                    })
                    .collect::<Delimited<',', _>>();

                write!(f, "*out = nullptr; *out = self->{name}({other_args});",
                    name = self.name(),
                )
            } else {
                // no "self"
                let args = if self.args.len() > 1 {
                    self.args[1..]
                        .iter()
                        .map(|a| {
                            fmt::from_fn(|f| {
                                write!(f, "{}", a.name)
                            })
                        })
                        .collect::<Delimited<',', _>>()
                } else {
                    Delimited::<',', _>::from_iter([])
                };

                write!(f, "self->{name}({args});",
                    name = self.name(),
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

impl ApiFnArg {
    fn self_arg(is_const: bool, t: &str) -> Self {
        Self {
            typ: ApiTyperef {
                is_const,
                typ: t.to_string(),
                relation: RefType::Reference,
            },
            name: "self".to_string(),
        }
    }

    fn out_arg(t: &str) -> Self {
        Self {
            typ: ApiTyperef {
                is_const: false,
                typ: t.to_string(),
                relation: RefType::OwnedReference,
            },
            name: "out".to_string(),
        }
    }
    
    fn nullcheck(&self, config: &AbiConfig) -> impl Display {
        fmt::from_fn(|f| {
            if let RefType::Pointer(depth) = self.typ.relation {
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
            }
            Ok(())
        })
    }
}

impl Display for ApiFnArg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{typ} {name}",
            typ = self.typ,
            name = self.name,
        )
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

impl Display for RefType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OwnedReference | Self::Pointer(2) => write!(f, "**"),
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