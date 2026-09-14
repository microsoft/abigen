use crate::{
    AbiConfig,
    query::{FnArg, FnArgType, FnDef, FnRetType},
    format::Delimited,
};
use std::hash::Hash;
use std::fmt::{self, Display};

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

impl ApiFnDef {
    pub fn header(&self, _config: &AbiConfig) -> impl Display {
        fmt::from_fn(|f| {
            fmt_proto(f, self)?;
            write!(f, ";")
        })
    }

    pub fn code(&self, config: &AbiConfig) -> impl Display {
        fmt::from_fn(|f| {
            fmt_proto(f, self)?;
            write!(f, "\n")?;
            fmt_body(f, self, config)
        })
    }

    pub fn pointer(&self, _config: &AbiConfig) -> impl Display {
        todo!()
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

impl Display for ApiTyperef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{const} {typ} {rel}",
            const = if self.is_const { "const" } else { "" },
            typ = self.typ,
            rel = self.relation,
        )
    }
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


fn fmt_proto(f: &mut std::fmt::Formatter<'_>, item: &ApiFnDef) -> std::fmt::Result {
    write!(f, "{ret} {name}({args})",
        ret = item.ret,
        name = item.name,
        args = item.args.iter().collect::<Delimited<',', _>>(),
    )
}

fn fmt_body(f: &mut std::fmt::Formatter<'_>, item: &ApiFnDef, config: &AbiConfig) -> std::fmt::Result {
    let nullchecks: Delimited<'\n', _> = item.args.iter()
        .map(|a| {
            fmt::from_fn(|f| {
                fmt_body_nullcheck(f, a, config)
            })
        }).collect();

    writeln!(f, r#"
        {{
            {nullchecks}
            {exec}
            return {success};
        }}"#,
        exec = fmt::from_fn(|f| {
            if item.noexcept {
                fmt_body_exec(f, item, config)
            } else {
                fmt_body_try_exec(f, item, config)
            }
        }),
        success = config.exception.success_value,
    )
}

fn fmt_body_nullcheck(f: &mut std::fmt::Formatter<'_>, item: &ApiFnArg, config: &AbiConfig) -> std::fmt::Result {
    if let RefType::Pointer(depth) = item.typ.relation {
        for i in 0..depth {
            write!(f, "if (")?;
            for _ in 0..i {
                write!(f, "*")?;
            }
            writeln!(f, "{name} == nullptr) {{ return {nullref_exception}; }}",
                name = item.name,
                nullref_exception = config.exception.nullref_value,
            )?;
        }
    }
    Ok(())
}

fn fmt_body_try_exec(f: &mut std::fmt::Formatter<'_>, item: &ApiFnDef, config: &AbiConfig) -> std::fmt::Result {
    let custom_exceptions: Delimited<'\n', _> = config.exception.custom_values.iter()
        .map(|(err, ret)| {
            fmt::from_fn(move |f| {
                write!(f, "catch ({err}) {{ return {ret}; }}")
            })
        }).collect();

    write!(f, r#"
        try {{ {exec} }}
        {custom_exceptions}
        catch (...) {{ return {generic_exception}; }}
    "#,
        exec = fmt::from_fn(|f| fmt_body_exec(f, item, config)),
        generic_exception = config.exception.generic_value,
    )
}

fn fmt_body_exec(f: &mut std::fmt::Formatter<'_>, item: &ApiFnDef, config: &AbiConfig) -> std::fmt::Result {
    if item.args[item.args.len()-1].name == "out" {
        // no "self", no "out"
        let other_args: Delimited<',', _> = item.args[1..item.args.len()-1].iter()
            .map(|a| {
                fmt::from_fn(|f| {
                    write!(f, "{}", a.name)
                })
            })
            .collect();

        write!(f, "*out = nullptr; *out = self->{name}({other_args});",
            name = item.name,
        )
    } else {
        // no "self"
        let args: Delimited<',', _> = item.args[1..].iter()
            .map(|a| {
                fmt::from_fn(|f| {
                    write!(f, "{}", a.name)
                })
            })
            .collect();

        write!(f, "self->{name}({args});",
            name = item.name,
        )
    }
}
