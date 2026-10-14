use crate::{
    AbiConfig, ApiFnArg, ApiType, TypeContext, TypeName, TypeRelation,
    format::{self, fn_wrap},
    name_concat,
    query::{FnDecl, FnDef},
};
use std::{
    cmp::Ordering,
    fmt,
    hash::{DefaultHasher, Hash, Hasher},
};

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
            (Self::Constructor, _) => Ordering::Less,
            (Self::Destructor, Self::Constructor) => Ordering::Greater,
            (Self::Destructor, Self::Destructor) => Ordering::Equal,
            (Self::Destructor, Self::Method { .. }) => Ordering::Less,
            (Self::Method { name: s_name, .. }, Self::Method { name: o_name, .. }) => {
                s_name.cmp(o_name)
            }
            (Self::Method { .. }, _) => Ordering::Greater,
        }
    }
}

impl AsRef<str> for ApiFnRole {
    fn as_ref(&self) -> &str {
        match self {
            Self::Constructor => "new",
            Self::Destructor => "destroy",
            Self::Method { name, .. } => name.as_str(),
        }
    }
}

impl PartialOrd for ApiFnRole {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl ApiFnRole {
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ApiFnDef {
    pub class_name: TypeName,

    pub role: ApiFnRole,

    pub args: Vec<ApiFnArg>,

    /// If this function expects a class instance reference, this will contain it.
    pub self_arg: Option<ApiFnArg>,

    /// If this function is expected to return a value, this will contain its type. May be applied either as a return
    /// value or an "out" parameter.
    pub out_arg: Option<ApiFnArg>,

    /// Whether or not this function needs exception handling.
    pub noexcept: bool,

    /// Whether the self reference is const or not.
    pub is_const: bool,
}

impl From<&FnDef::Match<'_>> for ApiFnDef {
    fn from(value: &FnDef::Match) -> Self {
        Self {
            role: ApiFnRole::Method {
                ret: ApiType::from(value),
                name: value.decl.name.as_str().to_string(),
            },
            args: value.decl.args.iter().map(|a| a.into()).collect(),
            noexcept: value.decl.noexcept.is_some(),
            is_const: value.decl.is_const.is_some(),
            ..Default::default()
        }
    }
}

impl From<&FnDecl::Match<'_>> for ApiFnDef {
    fn from(value: &FnDecl::Match<'_>) -> Self {
        Self {
            role: ApiFnRole::Constructor,
            args: value.args.iter().map(|a| a.into()).collect(),
            noexcept: value.noexcept.is_some(),
            is_const: value.is_const.is_some(),
            ..Default::default()
        }
    }
}

impl Ord for ApiFnDef {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.class_name
            .cmp(&other.class_name)
            .then(self.role.cmp(&other.role))
            .then(self.args.iter().cmp(other.args.iter()))
    }
}

impl PartialOrd for ApiFnDef {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Default for ApiFnDef {
    fn default() -> Self {
        Self {
            class_name: TypeName::new(),
            role: ApiFnRole::Method {
                name: "undefined".into(),
                ret: ApiType::void(),
            },
            args: vec![],
            self_arg: None,
            out_arg: None,
            noexcept: false,
            is_const: false,
        }
    }
}

impl ApiFnDef {
    pub fn header(
        &self,
        f: &mut fmt::Formatter<'_>,
        config: &AbiConfig,
        make_unique: bool,
    ) -> fmt::Result {
        self.proto(f, config, make_unique)?;
        write!(f, ";")
    }

    pub fn code(
        &self,
        f: &mut fmt::Formatter<'_>,
        config: &AbiConfig,
        make_unique: bool,
    ) -> fmt::Result {
        self.proto(f, config, make_unique)?;
        writeln!(f)?;
        self.body(f, config)
    }

    pub(crate) fn needs_trycatch(&self) -> bool {
        !self.noexcept
            || self.self_arg.is_some()
            || self
                .args
                .iter()
                .any(|a| matches!(a.typ.relation, TypeRelation::Ref | TypeRelation::Owned))
    }

    pub(crate) fn set_implicit_fields(&mut self, class_name: &TypeName) {
        self.class_name = class_name.clone();

        for a in self.args.iter_mut() {
            if class_name.ends_with(&a.typ.typ) {
                a.typ.typ = class_name.clone();
            }
        }
        if let ApiFnRole::Method { ret, .. } = &mut self.role
            && class_name.ends_with(&ret.typ)
        {
            ret.typ = class_name.clone();
        }

        self.self_arg = match &self.role {
            ApiFnRole::Constructor => None,
            ApiFnRole::Destructor => Some(ApiFnArg {
                typ: ApiType {
                    is_const: false,
                    typ: self.class_name.clone(),
                    relation: TypeRelation::Owned,
                },
                name: "self_ptr".into(),
                is_out: false,
            }),
            ApiFnRole::Method { .. } => Some(ApiFnArg {
                typ: ApiType {
                    is_const: self.is_const,
                    typ: self.class_name.clone(),
                    relation: TypeRelation::Ref,
                },
                name: "self_ptr".to_string(),
                is_out: false,
            }),
        };

        self.out_arg = match &self.role {
            ApiFnRole::Constructor => Some(ApiFnArg {
                typ: ApiType {
                    is_const: false,
                    typ: self.class_name.clone(),
                    relation: TypeRelation::Ref,
                },
                name: "out".into(),
                is_out: self.needs_trycatch(),
            }),
            ApiFnRole::Destructor => None,
            ApiFnRole::Method { ret, .. } if ret.is_void() => None,
            ApiFnRole::Method { ret, .. } => Some(ApiFnArg {
                typ: ApiType {
                    is_const: false,
                    typ: ret.typ.clone(),
                    relation: ret.relation,
                },
                name: "out".into(),
                is_out: self.needs_trycatch(),
            }),
        };
    }

    pub(crate) fn name_c(&self, f: &mut fmt::Formatter<'_>, make_unique: bool) -> fmt::Result {
        let mut h = DefaultHasher::new();
        match &self.role {
            ApiFnRole::Method { name, ret, .. } => {
                name.hash(&mut h);
                ret.hash(&mut h);
            }
            ApiFnRole::Constructor => {
                "ctor".hash(&mut h);
            }
            ApiFnRole::Destructor => {
                "dtor".hash(&mut h);
            }
        }
        for a in self.args.iter() {
            a.hash(&mut h);
        }
        let slug = format!("{:#x}", h.finish());
        let slug = (&slug[2..6]).to_string();

        if make_unique {
            name_concat!(self.class_name.iter(), [self.role.as_str(), slug.as_str()]).snake_case(f)
        } else {
            name_concat!(self.class_name.iter(), [self.role.as_str()]).snake_case(f)
        }
    }

    fn args_c(&self) -> impl Iterator<Item = &ApiFnArg> {
        self.self_arg
            .iter()
            .chain(self.args.iter())
            .chain(self.out_arg.iter().filter(|a| a.is_out))
    }

    fn proto(
        &self,
        f: &mut fmt::Formatter<'_>,
        config: &AbiConfig,
        make_unique: bool,
    ) -> fmt::Result {
        // return type
        if self.needs_trycatch() {
            write!(f, "{}", config.exception.value_type)?;
        } else if let ApiFnRole::Constructor = &self.role {
            ApiType {
                typ: self.class_name.clone(),
                is_const: self.is_const,
                relation: TypeRelation::Ptr(1),
            }
            .decl(f, config, TypeContext::Outbound)?;
        } else if let ApiFnRole::Method { ret, .. } = &self.role {
            ret.decl(f, config, TypeContext::Outbound)?;
        } else {
            write!(f, "void")?;
        }
        write!(f, " ")?;

        self.name_c(f, make_unique)?;
        write!(f, "(")?;
        format::delimited(
            f,
            ", ",
            self.args_c().map(|a| fn_wrap!(a.decl, config, true)),
        )?;
        write!(f, ")")
    }

    fn body(&self, f: &mut fmt::Formatter<'_>, config: &AbiConfig) -> fmt::Result {
        writeln!(f, "{{")?;

        // null checks
        for a in self.args_c() {
            a.nullcheck(f, config)?;
            writeln!(f)?;
        }
        writeln!(f)?;

        // self cast
        if let Some(mut self_arg) = self.self_arg.clone() {
            let mut self_ref = self_arg.clone();
            self_ref.name = "self".into();

            // pretend it's a reference to a pointer rather than a flat owned reference
            if self_arg.typ.relation == TypeRelation::Owned {
                self_arg.typ.relation = TypeRelation::Ref;
                self_ref.typ.relation = TypeRelation::Ptr(1);
            }

            self_ref.decl(f, config, false)?;
            write!(f, " = ")?;
            self_arg.deref(f, config, false)?;
            writeln!(f, ";")?;
        }

        // declare the return value if necessary
        // if let Some(out) = &self.out_arg && !out.is_out {
        //     out.decl(f, config, true)?;
        //     writeln!(f, ";")?;
        // }

        if self.needs_trycatch() {
            self.body_try_exec(f, config)?;
        } else {
            self.body_exec(f, config)?;
        }
        writeln!(f)?;

        if self.needs_trycatch() {
            writeln!(f, "return {};", config.exception.success_value)?;
        } else {
            writeln!(f, "return out;")?;
        }

        write!(f, "}}")
    }

    fn body_try_exec(&self, f: &mut fmt::Formatter<'_>, config: &AbiConfig) -> fmt::Result {
        if let Some(out) = &self.out_arg {
            out.deref(f, config, false)?;
            write!(f, " = ")?;
            out.typ.default_value(f, config)?;
            writeln!(f, ";")?;
        }

        write!(f, "try {{ ")?;
        self.body_exec(f, config)?;
        writeln!(f, " }}")?;

        for (catch, ret) in config.exception.custom_values.iter() {
            writeln!(f, "catch ({catch}) {{ return {ret}; }}")?;
        }

        writeln!(
            f,
            "catch (...) {{ return {}; }}",
            config.exception.generic_failure_value
        )
    }

    fn body_exec(&self, f: &mut fmt::Formatter<'_>, config: &AbiConfig) -> fmt::Result {
        let arg = |f: &mut fmt::Formatter<'_>, a: &ApiFnArg| a.deref(f, config, false);

        match (&self.role, &self.out_arg) {
            (ApiFnRole::Constructor, Some(o @ ApiFnArg { is_out: true, .. })) => {
                write!(
                    f,
                    "*out = {cast} new {name}({args});",
                    cast = fn_wrap!(o.typ.cast, config, TypeContext::Outbound),
                    name = fn_wrap!(self.class_name.cpp_case),
                    args = fn_wrap!(
                        format::delimited,
                        ", ",
                        self.args.iter().map(|a| fn_wrap!(arg, a))
                    ),
                )?;
            }
            (ApiFnRole::Constructor, Some(o @ ApiFnArg { is_out: false, .. })) => {
                write!(
                    f,
                    "{out} = {cast} new {name}({args});",
                    out = fn_wrap!(o.decl, config, true),
                    cast = fn_wrap!(o.typ.cast, config, TypeContext::Outbound),
                    name = fn_wrap!(self.class_name.cpp_case),
                    args = fn_wrap!(
                        format::delimited,
                        ", ",
                        self.args.iter().map(|a| fn_wrap!(arg, a))
                    ),
                )?;
            }
            (ApiFnRole::Destructor, _) => {
                writeln!(f, "delete self;")?;
                writeln!(f, "*self_ptr = nullptr;")?;
            }
            (ApiFnRole::Method { name, .. }, Some(o)) => {
                write!(
                    f,
                    "{out} = {deref_pre} self.{name}({args}){deref_post};",
                    out = fn_wrap!(o.deref, config, true),
                    deref_pre = fn_wrap!(o.typ.deref_pre_name, config, TypeContext::Outbound),
                    args = fn_wrap!(
                        format::delimited,
                        ", ",
                        self.args.iter().map(|a| fn_wrap!(arg, a))
                    ),
                    deref_post = fn_wrap!(o.typ.deref_post_name, config, TypeContext::Outbound),
                )?;
            }
            (ApiFnRole::Method { name, .. }, None) => {
                write!(f, "self.{}(", name)?;
                format::delimited(f, ", ", self.args.iter().map(|a| fn_wrap!(arg, a)))?;
                write!(f, ");")?;
            }
            _ => {
                return Err(fmt::Error);
            }
        }

        for a in &self.args {
            if a.typ.relation == TypeRelation::Owned {
                writeln!(f, "*{} = nullptr;", a.name)?;
            }
        }

        Ok(())
    }
}
