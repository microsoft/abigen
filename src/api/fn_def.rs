use crate::{
    AbiConfig, LANG, format::Delimited, query::{FnArg, FnArgType, FnDef, FnRetType}, repeat,
};
use std::{borrow::Cow, hash::Hash};
use std::{
    fmt::{self, Display},
    sync::LazyLock,
};

static K_CLASS: LazyLock<u16> = LazyLock::new(|| LANG.id_for_node_kind("class_specifier", true));
static K_NS: LazyLock<u16> = LazyLock::new(|| LANG.id_for_node_kind("namespace_definition", true));
static K_NESTED_NS: LazyLock<u16> =
    LazyLock::new(|| LANG.id_for_node_kind("nested_namespace_specifier", true));

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ApiFnDef {
    pub ret: ApiType,
    pub name: String,
    pub fqns: Vec<String>,
    pub args: Vec<ApiFnArg>,
    pub noexcept: bool,
    pub is_const: bool,
}

impl From<&FnDef::Match<'_>> for ApiFnDef {
    fn from(value: &FnDef::Match) -> Self {
        let mut fqns = vec![];
        let mut parent = value.decl.name;
        while let Some(p) = parent.parent() {
            parent = p;
            if parent.node.kind_id() == *K_CLASS
                && let Some(name) = parent.child_by_field_name("name")
            {
                // class definition
                fqns.insert(0, name.as_str().to_string());
            } else if parent.node.kind_id() == *K_NS
                && let Some(name) = parent.child_by_field_name("name")
            {
                // namespace definition
                if name.node.kind_id() == *K_NESTED_NS {
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
        let ours = self.fqns.iter().chain([&self.name].into_iter());
        let theirs = other.fqns.iter().chain([&other.name].into_iter());
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
        fmt::from_fn(|f| write!(f, "{};", self.proto(config)))
    }

    pub fn code(&self, config: &AbiConfig) -> impl Display {
        fmt::from_fn(|f| {
            write!(
                f,
                "{proto}\n{body}",
                proto = self.proto(config),
                body = self.body(config),
            )
        })
    }

    pub fn pointer(&self, _config: &AbiConfig) -> impl Display {
        todo!()
    }

    fn name_c(&self) -> impl Display {
        self.fqns
            .iter()
            .map(|p| p.to_lowercase())
            .chain([self.name.to_lowercase()].into_iter())
            .collect::<Delimited<'_'>>()
    }

    /// The name of the C type that opaquely stands in for the C++ type
    fn inst_handle_t(&self) -> impl Display {
        self.fqns.iter().collect::<Delimited<'_'>>()
    }

    fn args_c<'a>(&'a self) -> impl IntoIterator<Item = Cow<'a, ApiFnArg>> {
        let mut args: Vec<_> = self.args.iter().map(|a| Cow::Borrowed(a)).collect();

        let self_arg = ApiFnArg::self_arg(self.is_const, self.inst_handle_t().to_string());
        args.insert(0, Cow::Owned(self_arg));

        if let Some(o) = ApiFnArg::out_arg(&self.ret) {
            args.push(Cow::Owned(o));
        }

        args
    }

    fn proto(&self, config: &AbiConfig) -> impl Display {
        fmt::from_fn(|f| {
            write!(
                f,
                "{ret} {name}({args})",
                ret = config.exception.value_type,
                name = self.name_c(),
                args = self
                    .args_c()
                    .into_iter()
                    .map(|a| a.decl().to_string())
                    .collect::<Delimited<','>>(),
            )
        })
    }

    fn body(&self, config: &AbiConfig) -> impl Display {
        fmt::from_fn(|f| {
            writeln!(f, "{{")?;

            for a in self.args_c() {
                writeln!(f, "{}", a.nullcheck(config))?;
            }

            self.body_try_exec(config).fmt(f)?;

            writeln!(f, "return {};", config.exception.success_value)?;
            write!(f, "}}")
        })
    }

    fn body_try_exec(&self, config: &AbiConfig) -> impl Display {
        fmt::from_fn(|f| {
            if let Some(out) = ApiFnArg::out_arg(&self.ret) {
                writeln!(f, "{out} = {default};",
                    out = out.deref(),
                    default = out.typ.default_value(false))?;
            }

            writeln!(f, "try {{ {} }}", self.body_exec(config))?;

            for (catch, ret) in config.exception.custom_values.iter() {
                writeln!(f, "catch ({catch}) {{ return {ret}; }}")?;
            }

            write!(
                f,
                "catch (...) {{ return {}; }}",
                config.exception.generic_failure_value
            )
        })
    }

    fn body_exec(&self, _config: &AbiConfig) -> impl Display {
        fmt::from_fn(|f| {
            let args = self
                .args
                .iter()
                .map(|a| a.deref())
                .collect::<Delimited<','>>();

            if let Some(o) = ApiFnArg::out_arg(&self.ret) {
                write!(
                    f,
                    "{out} = self->{name}({args});",
                    out = o.deref(),
                    name = self.name,
                )
            } else {
                write!(f, "self->{}({args});", self.name)
            }
        })
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
    fn self_arg(is_const: bool, t: impl Into<String>) -> Self {
        Self {
            typ: ApiType {
                is_const,
                typ: t.into(),
                relation: TypeRelation::Ref,
            },
            name: "self".to_string(),
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

    fn nullcheck(&self, config: &AbiConfig) -> impl Display {
        fmt::from_fn(|f| {
            for i in 0..self.typ.relation.nullcheck_depth(self.is_out) {
                write!(f, "if ({deref}{name} == nullptr) {{ return {nullref_exception}; }}",
                    deref = repeat("*", i),
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
                typ = self.typ.decl(self.is_out),
                name = self.name,
            )
        })
    }

    fn deref(&self) -> impl Display {
        fmt::from_fn(|f| {
            write!(f, "{deref}{name}",
                deref = self.typ.relation.deref(self.is_out),
                name = self.name,
            )
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct ApiType {
    pub typ: String,
    pub is_const: bool,
    pub relation: TypeRelation,
}

impl From<&FnArgType::Match<'_>> for ApiType {
    fn from(value: &FnArgType::Match<'_>) -> Self {
        Self {
            is_const: value.q.is_some(),
            typ: value.t.as_str().to_string(),
            relation: TypeRelation::from(value),
        }
    }
}

impl From<&FnRetType::Match<'_>> for ApiType {
    fn from(value: &FnRetType::Match<'_>) -> Self {
        Self {
            is_const: value.q.is_some(),
            typ: value.t.as_str().to_string(),
            relation: TypeRelation::from(value),
        }
    }
}

impl ApiType {
    pub fn void() -> Self {
        Self {
            typ: "void".to_string(),
            is_const: false,
            relation: TypeRelation::Value,
        }
    }

    pub fn is_void(&self) -> bool {
        self.typ == "void"
    }

    pub fn decl(&self, is_out: bool) -> impl Display {
        fmt::from_fn(move |f| {
            write!(f, "{const} {typ} {rel}",
                const = if self.is_const { "const" } else { "" },
                typ = self.typ,
                rel = self.relation.decl(is_out),
            )
        })
    }

    pub fn default_value(&self, is_out: bool) -> impl Display {
        fmt::from_fn(move |f| {
            write!(f, "{typ} {rel} {{}}",
                typ = self.typ,
                rel = self.relation.decl(is_out),
            )
        })
    }
}

/// The level of indirection between a variable's real type and its dereferenced type
/// 
/// Each type can be serialized in four ways: input or output, and declaration or dereference. The full table is below:
/// 
/// | Variant | Decl In | Decl Out | Deref In | Deref Out |
/// |---------|---------|----------|----------|-----------|
/// | Value   | T       | T*       | T        | *T        |
/// | Ref     | T*      | T**      | *T       | *T        |
/// | Owned   | T**     | T**      | move(*T) | *T        |
/// | Ptr(n)  | T*(n)   | T*(n+1)  | T        | *T        |
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TypeRelation {
    #[default]
    Value,
    Ref,
    Owned,
    Ptr(usize),
}

impl From<&FnArgType::Match<'_>> for TypeRelation {
    fn from(value: &FnArgType::Match<'_>) -> Self {
        if value.r.len() == 0 {
            TypeRelation::Value
        } else if value.r[0].r.as_str() == "&" {
            TypeRelation::Ref
        } else if value.r[0].r.as_str() == "&&" {
            TypeRelation::Owned
        } else {
            TypeRelation::Ptr(value.r.len())
        }
    }
}

impl From<&FnRetType::Match<'_>> for TypeRelation {
    fn from(value: &FnRetType::Match<'_>) -> Self {
        if value.r.len() == 0 {
            TypeRelation::Value
        } else if value.r[0].r.as_str() == "&" {
            TypeRelation::Ref
        } else if value.r[0].r.as_str() == "&&" {
            TypeRelation::Owned
        } else {
            TypeRelation::Ptr(value.r.len())
        }
    }
}

impl TypeRelation {
    pub fn nullcheck_depth(&self, is_out: bool) -> usize {
        match self {
            Self::Value if is_out => 1,
            Self::Value => 0,
            Self::Ref if is_out => 1,
            Self::Ref => 1,
            Self::Owned if is_out => 1,
            Self::Owned => 2,
            Self::Ptr(_) if is_out => 1,
            Self::Ptr(_) => 0,
        }
    }

    pub fn decl(&self, is_out: bool) -> impl Display {
        fmt::from_fn(move |f| {
            match self {
                Self::Value if is_out => write!(f, "*"),
                Self::Value => Ok(()),
                Self::Ref if is_out => write!(f, "**"),
                Self::Ref => write!(f, "*"),
                Self::Owned if is_out => write!(f, "**"),
                Self::Owned => write!(f, "**"),
                Self::Ptr(depth) if is_out => write!(f, "{}", repeat("*", depth+1)),
                Self::Ptr(depth) => write!(f, "{}", repeat("*", *depth)),
            }
        })
    }

    pub fn deref(&self, is_out: bool) -> impl Display {
        fmt::from_fn(move |f| {
            match self {
                Self::Value if is_out => write!(f, "*"),
                Self::Value => Ok(()),
                Self::Ref if is_out => write!(f, "*"),
                Self::Ref => write!(f, "*"),
                Self::Owned if is_out => write!(f, "*"),
                Self::Owned => write!(f, "*"),
                Self::Ptr(_) if is_out => write!(f, "*"),
                Self::Ptr(_) => Ok(()),
            }
        })
    }
}