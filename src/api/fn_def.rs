use crate::{
    AbiConfig, ApiType, TypeContext, TypeName, TypeRelation,
    format::{self, fmt_wrap},
    name_concat,
    query::{FnArg, FnDef, f, k},
};
use std::{borrow::Cow, cmp::Ordering, fmt::{self, Display}, hash::Hash};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ApiFnRole {
    Constructor,
    Destructor,
    Method { name: String, ret: ApiType },
}

impl Ord for ApiFnRole {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Constructor, Self::Constructor) => Ordering::Equal,
            (Self::Constructor, _) => Ordering::Greater,
            (Self::Destructor, Self::Constructor) => Ordering::Less,
            (Self::Destructor, Self::Destructor) => Ordering::Equal,
            (Self::Destructor, Self::Method { .. }) => Ordering::Greater,
            (Self::Method { name: s_name, .. }, Self::Method { name: o_name, .. }) => {
                s_name.cmp(o_name)
            },
            (Self::Method { .. }, _) => Ordering::Less,
        }
    }
}

impl PartialOrd for ApiFnRole {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ApiFnDef {
    pub role: ApiFnRole,
    pub fqns: TypeName,
    pub args: Vec<ApiFnArg>,
    pub noexcept: bool,
    pub is_const: bool,
}

impl From<&FnDef::Match<'_>> for ApiFnDef {
    fn from(value: &FnDef::Match) -> Self {
        let mut fqns = TypeName::new();
        let mut parent = value.decl.name;
        while let Some(p) = parent.parent() {
            parent = p;
            if parent.node.kind_id() == *k::CLASS
                && let Some(name) = parent.child_by_field_id(*f::NAME)
            {
                // class definition
                fqns.insert(0, name.as_str().to_string());
            } else if parent.node.kind_id() == *k::NS
                && let Some(name) = parent.child_by_field_id(*f::NAME)
            {
                // namespace definition
                if name.node.kind_id() == *k::NESTED_NS {
                    // multi-part namespace name, i.e. x::y
                    for (i, part) in name.named_children(&mut name.walk()).enumerate() {
                        fqns.insert(i, part.as_str().to_string());
                    }
                } else {
                    // single-part namespace name
                    fqns.insert(0, name.as_str().to_string());
                }
            }
        }

        Self {
            role: ApiFnRole::Method {
                ret: ApiType::from(&value.ret),
                name: value.decl.name.as_str().to_string(),
            },
            fqns,
            args: value.decl.args.iter().map(|a| a.into()).collect(),
            noexcept: value.decl.noexcept.is_some(),
            is_const: value.decl.immut.is_some(),
        }
    }
}

impl Ord for ApiFnDef {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.fqns.iter().cmp(other.fqns.iter()).then(self.role.cmp(&other.role))
    }
}

impl PartialOrd for ApiFnDef {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl ApiFnDef {
    pub fn header(&self, config: &AbiConfig) -> impl Display {
        fmt::from_fn(|f| {
            self.proto(f, config)?;
            write!(f, ";")
        })
    }

    pub fn code(&self, config: &AbiConfig) -> impl Display {
        fmt::from_fn(|f| {
            self.proto(f, config)?;
            writeln!(f)?;
            self.body(f, config)
        })
    }

    pub fn pointer(&self, _config: &AbiConfig) -> impl Display {
        todo!()
    }

    fn name_c(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match &self.role {
            ApiFnRole::Constructor => "new",
            ApiFnRole::Destructor => "destroy",
            ApiFnRole::Method { name, .. } => name.as_str(),
        };
        name_concat!(self.fqns.iter(), [name]).snake_case(f)
    }

    fn args_c<'a>(&'a self) -> impl Iterator<Item = Cow<'a, ApiFnArg>> {
        let mut args: Vec<_> = self.args.iter().map(|a| Cow::Borrowed(a)).collect();

        if let Some(self_arg) = ApiFnArg::self_arg(self) {
            args.insert(0, Cow::Owned(self_arg));
        }

        if let Some(o) = ApiFnArg::out_arg(self) {
            args.push(Cow::Owned(o));
        }

        args.into_iter()
    }

    fn proto(&self, f: &mut fmt::Formatter<'_>, config: &AbiConfig) -> fmt::Result {
        if matches!(self.role, ApiFnRole::Constructor) 
            && self.noexcept 
            && self.args.iter().all(|a| {
                a.typ.relation != TypeRelation::Ref && a.typ.relation != TypeRelation::Owned
            })
        {
            ApiType {
                typ: self.fqns.clone(),
                is_const: false,
                relation: TypeRelation::Owned,
            }.decl(f, config, TypeContext::Outbound)?;
            write!(f, " ")?;
        } else {
            write!(f, "{} ", config.exception.value_type)?;
        }

        self.name_c(f)?;
        write!(f, "(")?;
        format::delimited(
            f,
            ", ",
            self.args_c().map(|a| fmt_wrap!(a.decl, config, true)),
        )?;
        write!(f, ")")
    }

    fn body(&self, f: &mut fmt::Formatter<'_>, config: &AbiConfig) -> fmt::Result {
        writeln!(f, "{{")?;

        for a in self.args_c() {
            if a.nullcheck(f, config).is_ok() {
                writeln!(f)?;
            } else {
                return Err(fmt::Error);
            }
        }
        writeln!(f)?;

        writeln!(
            f,
            "{inner_t}& self = *({inner_t}*)self_ptr;",
            inner_t = fmt::from_fn(|f| {
                if self.is_const {
                    write!(f, "const ")?;
                }
                self.fqns.cpp_case(f)
            }),
        )?;

        self.body_try_exec(f, config)?;
        writeln!(f)?;

        writeln!(f, "return {};", config.exception.success_value)?;
        write!(f, "}}")
    }

    fn body_try_exec(&self, f: &mut fmt::Formatter<'_>, config: &AbiConfig) -> fmt::Result {
        if let Some(out) = ApiFnArg::out_arg(self) {
            out.deref(f, config, false)?;
            write!(f, " = ")?;
            out.typ.default_value(f, config)?;
            writeln!(f, ";")?;
        }

        writeln!(f, "try {{ {} }}", fmt_wrap!(self.body_exec, config))?;

        for (catch, ret) in config.exception.custom_values.iter() {
            writeln!(f, "catch ({catch}) {{ return {ret}; }}")?;
        }

        write!(
            f,
            "catch (...) {{ return {}; }}",
            config.exception.generic_failure_value
        )
    }

    fn body_exec(&self, f: &mut fmt::Formatter<'_>, config: &AbiConfig) -> fmt::Result {
        let arg = |f: &mut fmt::Formatter<'_>, a: &ApiFnArg| {
            a.deref(f, config, false)
        };

        match &self.role {
            ApiFnRole::Constructor => {
                Ok(())
            },
            ApiFnRole::Destructor => {
                Ok(())
            },
            ApiFnRole::Method { name, .. } => {
                if let Some(o) = ApiFnArg::out_arg(self) {
                    o.deref(f, config, true)?;
                    write!(f, " = ")?;
                    o.typ.deref_pre_name(f, config, TypeContext::Outbound)?;
                    write!(f, "self.{}(", name)?;
                    format::delimited(f, ", ", self.args.iter().map(|a| fmt_wrap!(arg, a)))?;
                    write!(f, ")")?;
                    o.typ.deref_post_name(f, config, TypeContext::Outbound)?;
                    write!(f, ";")
                } else {
                    write!(f, "self.{}(", name)?;
                    format::delimited(f, ", ", self.args.iter().map(|a| fmt_wrap!(arg, a)))?;
                    write!(f, ");")
                }
            },
        }
        
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ApiFnArg {
    pub typ: ApiType,
    pub name: String,
    pub is_out: bool,
}

impl From<&FnArg::Match<'_>> for ApiFnArg {
    fn from(value: &FnArg::Match) -> Self {
        Self {
            typ: ApiType::from(&value.typ),
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
    fn self_arg(def: &ApiFnDef) -> Option<Self> {
        match &def.role {
            ApiFnRole::Constructor => None,
            ApiFnRole::Destructor => Some(Self {
                typ: ApiType {
                    is_const: false,
                    typ: def.fqns.clone(),
                    relation: TypeRelation::Owned,
                },
                name: "self_ptr".into(),
                is_out: false,
            }),
            ApiFnRole::Method { .. } => Some(Self {
                typ: ApiType {
                    is_const: def.is_const,
                    typ: def.fqns.clone(),
                    relation: TypeRelation::Ref,
                },
                name: "self_ptr".to_string(),
                is_out: false,
            }),
        }
    }

    fn out_arg(def: &ApiFnDef) -> Option<Self> {
        match &def.role {
            ApiFnRole::Constructor if def.noexcept => None,
            ApiFnRole::Constructor => {
                Some(Self {
                    typ: ApiType {
                        is_const: false,
                        typ: def.fqns.clone(),
                        relation: TypeRelation::Owned,
                    },
                    name: "out".into(),
                    is_out: true,
                })
            },
            ApiFnRole::Destructor => None,
            ApiFnRole::Method { ret, .. } if ret.is_void() => None,
            ApiFnRole::Method { ret, .. } => {
                Some(Self {
                    typ: ApiType {
                        is_const: false,
                        typ: ret.typ.clone(),
                        relation: ret.relation,
                    },
                    name: "out".into(),
                    is_out: true,
                })
            },
        }
    }

    fn nullcheck(&self, f: &mut fmt::Formatter<'_>, config: &AbiConfig) -> fmt::Result {
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
                deref = fmt_wrap!(format::repeat, "*", i),
                name = self.name,
                nullref_exception = config.exception.nullref_value,
            )?;
            if i < depth - 1 {
                writeln!(f)?;
            }
        }
        Ok(())
    }

    fn decl(
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
    fn deref(
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
