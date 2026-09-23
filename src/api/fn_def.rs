use crate::{
    AbiConfig, ApiClassDef, ApiType, TypeContext, TypeRelation,
    format::{self, fmt_wrap},
    name_concat,
    query::{FnArg, FnDef, FnDecl},
};
use std::{borrow::Cow, cmp::Ordering, fmt, hash::{DefaultHasher, Hash, Hasher}};

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
    pub fn name_matches(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Constructor, Self::Constructor) => true,
            (Self::Destructor, Self::Destructor) => true,
            (Self::Method { name: s_name, ..}, Self::Method { name: o_name, ..}) => s_name == o_name,
            _ => false,
        }
    }

    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ApiFnDef {
    pub role: ApiFnRole,
    pub args: Vec<ApiFnArg>,
    pub noexcept: bool,
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
        }
    }
}

impl Ord for ApiFnDef {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.role.cmp(&other.role)
    }
}

impl PartialOrd for ApiFnDef {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl ApiFnDef {
    pub fn header(
        &self,
        f: &mut fmt::Formatter<'_>,
        config: &AbiConfig,
        class: &ApiClassDef,
        make_unique: bool,
    ) -> fmt::Result {
        self.proto(f, config, class, make_unique)?;
        write!(f, ";")
    }

    pub fn code(
        &self,
        f: &mut fmt::Formatter<'_>,
        config: &AbiConfig,
        class: &ApiClassDef,
        make_unique: bool,
    ) -> fmt::Result {
        self.proto(f, config, class, make_unique)?;
        writeln!(f)?;
        self.body(f, config, class)
    }

    fn name_c(&self, f: &mut fmt::Formatter<'_>, class: &ApiClassDef, make_unique: bool) -> fmt::Result {
        let mut h = DefaultHasher::new();
        match &self.role {
            ApiFnRole::Method { name, ret, .. } => {
                name.hash(&mut h);
                ret.hash(&mut h);
            },
            ApiFnRole::Constructor => {
                "ctor".hash(&mut h);
            },
            ApiFnRole::Destructor => {
                "dtor".hash(&mut h);
            },
        }
        for a in self.args.iter() {
            a.hash(&mut h);
        }
        let slug = format!("{:#x}", h.finish());
        let slug = (&slug[2..6]).to_string();

        if make_unique {
            name_concat!(class.name.iter(), [self.role.as_str(), slug.as_str()]).snake_case(f)
        } else {
            name_concat!(class.name.iter(), [self.role.as_str()]).snake_case(f)
        }
    }

    fn args_c<'a>(&'a self, class: &ApiClassDef) -> impl Iterator<Item = Cow<'a, ApiFnArg>> {
        let mut args: Vec<_> = self.args.iter().map(|a| Cow::Borrowed(a)).collect();

        if let Some(self_arg) = ApiFnArg::self_arg(class, self) {
            args.insert(0, Cow::Owned(self_arg));
        }

        if let Some(o) = ApiFnArg::out_arg(class, self) {
            args.push(Cow::Owned(o));
        }

        args.into_iter()
    }

    fn proto(
        &self,
        f: &mut fmt::Formatter<'_>,
        config: &AbiConfig,
        class: &ApiClassDef,
        make_unique: bool,
    ) -> fmt::Result {
        if matches!(self.role, ApiFnRole::Constructor)
            && self.noexcept
            && self.args.iter().all(|a| {
                a.typ.relation != TypeRelation::Ref && a.typ.relation != TypeRelation::Owned
            })
        {
            ApiType {
                typ: class.name.clone(),
                is_const: false,
                relation: TypeRelation::Ptr(1),
            }
            .decl(f, config, TypeContext::Outbound)?;
            write!(f, " ")?;
        } else {
            write!(f, "{} ", config.exception.value_type)?;
        }

        self.name_c(f, class, make_unique)?;
        write!(f, "(")?;
        format::delimited(
            f,
            ", ",
            self.args_c(class).map(|a| fmt_wrap!(a.decl, config, true)),
        )?;
        write!(f, ")")
    }

    fn body(
        &self,
        f: &mut fmt::Formatter<'_>,
        config: &AbiConfig,
        class: &ApiClassDef,
    ) -> fmt::Result {
        writeln!(f, "{{")?;

        for a in self.args_c(class) {
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
                class.name.cpp_case(f)
            }),
        )?;

        self.body_try_exec(f, config, class)?;
        writeln!(f)?;

        writeln!(f, "return {};", config.exception.success_value)?;
        write!(f, "}}")
    }

    fn body_try_exec(
        &self,
        f: &mut fmt::Formatter<'_>,
        config: &AbiConfig,
        class: &ApiClassDef,
    ) -> fmt::Result {
        if let Some(out) = ApiFnArg::out_arg(class, self) {
            out.deref(f, config, false)?;
            write!(f, " = ")?;
            out.typ.default_value(f, config)?;
            writeln!(f, ";")?;
        }

        writeln!(f, "try {{ {} }}", fmt_wrap!(self.body_exec, config, class))?;

        for (catch, ret) in config.exception.custom_values.iter() {
            writeln!(f, "catch ({catch}) {{ return {ret}; }}")?;
        }

        write!(
            f,
            "catch (...) {{ return {}; }}",
            config.exception.generic_failure_value
        )
    }

    fn body_exec(
        &self,
        f: &mut fmt::Formatter<'_>,
        config: &AbiConfig,
        class: &ApiClassDef,
    ) -> fmt::Result {
        let arg = |f: &mut fmt::Formatter<'_>, a: &ApiFnArg| a.deref(f, config, false);

        match &self.role {
            ApiFnRole::Constructor => Ok(()),
            ApiFnRole::Destructor => Ok(()),
            ApiFnRole::Method { name, .. } => {
                if let Some(o) = ApiFnArg::out_arg(class, self) {
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
            }
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
    fn self_arg(class_def: &ApiClassDef, method_def: &ApiFnDef) -> Option<Self> {
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

    fn out_arg(class_def: &ApiClassDef, method_def: &ApiFnDef) -> Option<Self> {
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
                is_out: true,
            }),
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
