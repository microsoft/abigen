use crate::{AbiConfig, TypeName, format, query};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct ApiType {
    pub typ: TypeName,
    pub is_const: bool,
    pub relation: TypeRelation,
}

impl From<&query::FnArgType::Match<'_>> for ApiType {
    fn from(value: &query::FnArgType::Match<'_>) -> Self {
        Self {
            is_const: value.q.is_some(),
            typ: value.t.try_into().unwrap(),
            relation: TypeRelation::from(value),
        }
    }
}

impl From<&query::FnRetType::Match<'_>> for ApiType {
    fn from(value: &query::FnRetType::Match<'_>) -> Self {
        Self {
            is_const: value.q.is_some(),
            typ: value.t.try_into().unwrap(),
            relation: TypeRelation::from(value),
        }
    }
}

impl ApiType {
    pub fn void() -> Self {
        Self {
            typ: TypeName::void().clone(),
            is_const: false,
            relation: TypeRelation::Value,
        }
    }

    pub fn is_void(&self) -> bool {
        &self.typ == TypeName::void()
    }

    /// Writes the declaration for this type, suitable for function argument or variable declaration.
    /// 
    /// ## Arguments
    /// * `is_out` - Whether this type is for an output argument. Output arguments get an extra layer of indirection.
    /// * `is_external` - If this type name needs to be C-safe, i.e. no namespaces or templates
    pub fn decl(&self, f: &mut fmt::Formatter<'_>, config: &AbiConfig, is_out: bool, is_external: bool) -> fmt::Result {
        if self.is_const {
            write!(f, "const ")?;
        }
        self.typ_c(f, config, is_external)?;
        self.relation.decl(f, is_out)
    }

    /// Use a variable of this type. Does not include the actual variable name, but will include any casts, derefs,
    /// or address-ofs to come up with the right type to match the request.
    /// 
    /// ## Arguments
    /// * `is_out` - Whether the variable being used is an output argument. Output arguments need an extra layer of
    ///     dereferencing.
    /// * `is_external` - Whether the result of this deref is for consumption by the C layer (external)
    ///     or the C++ layer (internal).
    pub fn deref(&self, f: &mut fmt::Formatter<'_>, config: &AbiConfig, is_out: bool, is_external: bool) -> fmt::Result {
        todo!()
    }

    pub fn default_value(&self, f: &mut fmt::Formatter<'_>, config: &AbiConfig) -> fmt::Result {
        if let TypeRelation::Ref | TypeRelation::Ptr(_) = &self.relation {
            write!(f, "nullptr")
        } else {
            self.typ_c(f, config, true)?;
            self.relation.decl(f, false)?;
            write!(f, "{{}}")
        }
    }

    /// If this type is not cast-mapped via [crate::AbiTypeConfig::cast_mapped], it does nothing.
    /// Otherwise, writes a cast expression.
    /// If `is_external` is set, casts from the original type to the mapped type. Otherwise, casts from mapped to original.
    pub fn cast(&self, f: &mut fmt::Formatter<'_>, config: &AbiConfig, is_external: bool) -> fmt::Result {
        if config.types.cast_mapped.contains_key(&self.typ) {
            write!(f, "(")?;
            self.typ_c(f, config, is_external)?;
            self.relation.decl(f, false)?;
            write!(f, ")")?;
            if let TypeRelation::Ref = &self.relation {
                write!(f, "&")?;
            }
        }
        Ok(())
    }

    /// Writes the underlying type name [Self::typ]. If `is_external` is set and this type is cast-mapped 
    /// via [crate::AbiTypeConfig::cast_mapped], writes the mapped type instead.
    fn typ_c(&self, f: &mut fmt::Formatter<'_>, config: &AbiConfig, is_external: bool) -> fmt::Result {
        if config.types.builtin.contains(&self.typ) {
            self.typ.cpp_case(f)
        } else if is_external && let Some(t) = config.types.cast_mapped.get(&self.typ) {
            t.cpp_case(f)
        } else if is_external && let Some(t) = config.types.resolved.get(&self.typ) {
            t.pascal_case(f)
        } else if is_external {
            self.typ.pascal_case(f)
        } else {
            self.typ.cpp_case(f)
        }
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

impl From<&query::FnArgType::Match<'_>> for TypeRelation {
    fn from(value: &query::FnArgType::Match<'_>) -> Self {
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

impl From<&query::FnRetType::Match<'_>> for TypeRelation {
    fn from(value: &query::FnRetType::Match<'_>) -> Self {
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

    pub fn decl(&self, f: &mut fmt::Formatter<'_>, is_out: bool) -> fmt::Result {
        match self {
            Self::Value if is_out => write!(f, "*"),
            Self::Value => Ok(()),
            Self::Ref if is_out => write!(f, "**"),
            Self::Ref => write!(f, "*"),
            Self::Owned if is_out => write!(f, "**"),
            Self::Owned => write!(f, "**"),
            Self::Ptr(depth) if is_out => format::repeat(f, "*", depth + 1),
            Self::Ptr(depth) => format::repeat(f, "*", *depth),
        }
    }

    pub fn deref(&self, f: &mut fmt::Formatter<'_>, is_out: bool) -> fmt::Result {
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
    }
}

#[cfg(test)]
mod test {
    macro_rules! assert_type (
        ([$func:ident, $typ:literal, $rel:ident, in, internal], $expected:literal) => {
            assert_type!([$func, $typ, $rel, false, false], $expected)
        };
        ([$func:ident, $typ:literal, $rel:ident, in, external], $expected:literal) => {
            assert_type!([$func, $typ, $rel, false, true], $expected)
        };
        ([$func:ident, $typ:literal, $rel:ident, out, internal], $expected:literal) => {
            assert_type!([$func, $typ, $rel, true, false], $expected)
        };
        ([$func:ident, $typ:literal, $rel:ident, out, external], $expected:literal) => {
            assert_type!([$func, $typ, $rel, true, true], $expected)
        };
        ([$func:ident, $typ:literal, $rel:ident, $is_out:literal, $is_external:literal], $expected:literal) => {
            assert_eq!(
                std::fmt::from_fn(|f| {
                    crate::ApiType {
                        typ: $typ.into(),
                        is_const: false,
                        relation: $crate::TypeRelation::$rel,
                    }.$func(f, &$crate::test::CONFIG, $is_out, $is_external)
                }).to_string(),
                $expected,
            )
        };
    );

    #[test]
    fn deref() {
        assert_type!([deref, "a::b", Value,  in, internal], "(a::b)");
        assert_type!([deref, "a::b", Value,  in, external], "");
        assert_type!([deref, "a::b", Value, out, internal], "*");
        assert_type!([deref, "a::b", Value, out, external], "*");
    }
}