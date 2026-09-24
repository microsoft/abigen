use crate::{
    AbiConfig, ApiClassDef, ApiFnArg, ApiType, TypeContext, TypeRelation,
    format::{self, fmt_wrap},
    name_concat,
    query::{FnDef, FnDecl},
};
use std::{cmp::Ordering, fmt, hash::{DefaultHasher, Hash, Hasher}};

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
        println!("ApiFnDef from FnDef");
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
        println!("ApiFnDef from FnDecl");
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
        self.role.cmp(&other.role)
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
            role: ApiFnRole::Method { name: "undefined".into(), ret: ApiType::void() },
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
        class: &ApiClassDef,
        make_unique: bool,
    ) -> fmt::Result {
        println!("ApiFnDef::header");
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

    pub(crate) fn needs_trycatch(&self) -> bool {
        !self.noexcept || self.self_arg.is_some() || self.args.iter().any(|a| {
            matches!(a.typ.relation, TypeRelation::Ref | TypeRelation::Owned)
        })
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

    fn args_c(&self) -> impl Iterator<Item = &ApiFnArg> {
        self.self_arg.iter().chain(self.args.iter()).chain(self.out_arg.iter())
    }

    fn proto(
        &self,
        f: &mut fmt::Formatter<'_>,
        config: &AbiConfig,
        class: &ApiClassDef,
        make_unique: bool,
    ) -> fmt::Result {
        if self.needs_trycatch() {
            write!(f, "{}", config.exception.value_type)?;
        } else if let ApiFnRole::Constructor = &self.role {
            ApiType {
                typ: class.name.clone(),
                is_const: false,
                relation: TypeRelation::Ptr(1),
            }
            .decl(f, config, TypeContext::Outbound)?;
        } else if let ApiFnRole::Method { ret, .. } = &self.role {
            ret.decl(f, config, TypeContext::Outbound)?;
        } else {
            write!(f, "void")?;
        }
        write!(f, " ")?;

        self.name_c(f, class, make_unique)?;
        write!(f, "(")?;
        format::delimited(
            f,
            ", ",
            self.args_c().map(|a| fmt_wrap!(a.decl, config, true)),
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
        if let Some(out) = &self.out_arg && !out.is_out {
            out.decl(f, config, true)?;
            writeln!(f)?;
        }

        if self.needs_trycatch() {
            self.body_try_exec(f, config, class)?;
        } else {
            self.body_exec(f, config, class)?;
        }
        writeln!(f)?;



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

        writeln!(
            f,
            "catch (...) {{ return {}; }}",
            config.exception.generic_failure_value
        )?;

        write!(f, "return {};", config.exception.success_value)
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
