use crate::{
    AbiConfig, TypeName, ApiType, TypeRelation,
    format::{self, fmt_wrap},
    name_concat,
    query::{k, f, FnArg, FnDef},
};
use std::{borrow::Cow, hash::Hash};
use std::{
    fmt::{self, Display},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ApiFnDef {
    pub ret: ApiType,
    pub name: String,
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
            ret: ApiType::from(&value.ret),
            name: value.decl.name.as_str().to_string(),
            fqns,
            args: value.decl.args.iter().map(|a| a.into()).collect(),
            noexcept: value.decl.noexcept.is_some(),
            is_const: value.decl.immut.is_some(),
        }
    }
}

impl Ord for ApiFnDef {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let ours = self.fqns.iter().chain([self.name.as_str()].into_iter());
        let theirs = other.fqns.iter().chain([other.name.as_str()].into_iter());
        ours.cmp(theirs)
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
        name_concat!(self.fqns.iter(), [self.name.as_str()]).snake_case(f)
    }

    /// The name of the C type that opaquely stands in for the C++ type
    fn inst_handle_t(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fqns.pascal_case(f)
    }

    fn args_c<'a>(&'a self) -> impl Iterator<Item = Cow<'a, ApiFnArg>> {
        let mut args: Vec<_> = self.args.iter().map(|a| Cow::Borrowed(a)).collect();

        let self_t = fmt::from_fn(|f| self.inst_handle_t(f)).to_string();
        let self_arg = ApiFnArg::self_arg(self.is_const, self_t);
        args.insert(0, Cow::Owned(self_arg));

        if let Some(o) = ApiFnArg::out_arg(&self.ret) {
            args.push(Cow::Owned(o));
        }

        args.into_iter()
    }

    fn proto(&self, f: &mut fmt::Formatter<'_>, config: &AbiConfig) -> fmt::Result {
        write!(f, "{} ", config.exception.value_type)?;
        self.name_c(f)?;
        write!(f, "(")?;
        format::delimited(f, ", ", self.args_c().map(|a| fmt_wrap!(a.decl, config, true)))?;
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

        writeln!(f, "{inner_t}& self = *({inner_t}*)self_ptr;",
            inner_t = fmt::from_fn(|f| self.fqns.cpp_case(f)),
        )?;

        self.body_try_exec(f, config)?;
        writeln!(f)?;

        writeln!(f, "return {};", config.exception.success_value)?;
        write!(f, "}}")
    }

    fn body_try_exec(&self, f: &mut fmt::Formatter<'_>, config: &AbiConfig) -> fmt::Result {
        if let Some(out) = ApiFnArg::out_arg(&self.ret) {
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
            a.typ.cast(f, config, false)?;
            a.deref(f, config, false)
        };

        if let Some(o) = ApiFnArg::out_arg(&self.ret) {
            o.deref(f, config, false)?;
            write!(f, " = ")?;
            o.typ.cast(f, config, true)?;
            write!(f, "self.{}(", self.name)?;
            format::delimited(f, ", ", self.args.iter().map(|a| fmt_wrap!(arg, a)))?;
            write!(f, ");")
        } else {
            write!(f, "self.{}(", self.name)?;
            format::delimited(f, ", ", self.args.iter().map(|a| fmt_wrap!(arg, a)))?;
            write!(f, ");")
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
    fn self_arg(is_const: bool, t: impl Into<TypeName>) -> Self {
        Self {
            typ: ApiType {
                is_const,
                typ: t.into(),
                relation: TypeRelation::Ref,
            },
            name: "self_ptr".to_string(),
            is_out: false,
        }
    }

    fn out_arg(t: &ApiType) -> Option<Self> {
        if !t.is_void() {
            Some(Self {
                typ: ApiType {
                    is_const: false,
                    typ: t.typ.clone(),
                    relation: t.relation,
                },
                name: "out".to_string(),
                is_out: true,
            })
        } else {
            None
        }
    }

    fn nullcheck(&self, f: &mut fmt::Formatter<'_>, config: &AbiConfig) -> fmt::Result {
        let depth = self.typ.relation.nullcheck_depth(self.is_out);
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

    fn decl(&self, f: &mut fmt::Formatter<'_>, config: &AbiConfig, is_external: bool) -> fmt::Result {
        self.typ.decl(f, config, self.is_out, is_external)?;
        write!(f, " {}", self.name)
    }

    /// Write 
    fn deref(&self, f: &mut fmt::Formatter<'_>, config: &AbiConfig, is_external: bool) -> fmt::Result {
        self.typ.deref(f, config, self.is_out, is_external)?;
        write!(f, "{}", self.name)
    }
}
