use crate::{AbiConfig, TypeName, format, query};
use std::{fmt, sync::LazyLock};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TypeContext {
    /// This is the type of an output argument
    OutArg,
    /// This type needs to be C-safe
    Outbound,
    /// This needs to be serialized as the original C++ type
    Inbound,
}

static VOID_T: LazyLock<ApiType> = LazyLock::new(|| {
    ApiType {
        typ: TypeName::void().clone(),
        is_const: false,
        relation: TypeRelation::Value,
    }
});

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ApiType {
    pub typ: TypeName,
    pub is_const: bool,
    pub relation: TypeRelation,
}

impl From<&query::FnArg::Match<'_>> for ApiType {
    fn from(value: &query::FnArg::Match<'_>) -> Self {
        Self {
            is_const: value.typ_qual.is_some(),
            typ: value.typ.try_into().unwrap(),
            relation: TypeRelation::from(value),
        }
    }
}

impl From<&query::FnDef::Match<'_>> for ApiType {
    fn from(value: &query::FnDef::Match<'_>) -> Self {
        Self {
            is_const: value.ret_qual.is_some(),
            typ: value.ret_typ.try_into().unwrap(),
            relation: TypeRelation::from(value),
        }
    }
}

impl Default for ApiType {
    fn default() -> Self {
        VOID_T.clone()
    }
}

impl ApiType {
    pub fn void() -> Self {
        VOID_T.clone()
    }

    pub fn is_void(&self) -> bool {
        *self == *VOID_T
    }

    /// Writes the declaration for this type, suitable for function argument or variable declaration.
    pub(crate) fn decl(
        &self,
        f: &mut fmt::Formatter<'_>,
        config: &AbiConfig,
        context: TypeContext,
    ) -> fmt::Result {
        if self.is_const {
            write!(f, "const ")?;
        }
        self.typename(f, config, context)?;
        self.relation.decl(f, context)
    }

    /// The part of the deref that comes before the variable name
    pub(crate) fn deref_pre_name(
        &self,
        f: &mut fmt::Formatter<'_>,
        config: &AbiConfig,
        context: TypeContext,
    ) -> fmt::Result {
        if self.relation == TypeRelation::Owned && context == TypeContext::Inbound {
            write!(f, "std::move(")?;
        }

        self.relation.deref_pre_cast(f, context)?;
        self.cast(f, config, context)?;
        self.relation.deref_post_cast(f, context)?;

        if self.relation == TypeRelation::Owned && context == TypeContext::Outbound {
            write!(f, "std::move(")?;
        }

        Ok(())
    }

    /// The part of the deref that comes after the variable name
    pub(crate) fn deref_post_name(
        &self,
        f: &mut fmt::Formatter<'_>,
        _config: &AbiConfig,
        context: TypeContext,
    ) -> fmt::Result {
        if self.relation == TypeRelation::Owned && context != TypeContext::OutArg {
            write!(f, ")")?;
        }
        Ok(())
    }

    pub(crate) fn default_value(
        &self,
        f: &mut fmt::Formatter<'_>,
        config: &AbiConfig,
    ) -> fmt::Result {
        if let TypeRelation::Ref | TypeRelation::Ptr(_) = &self.relation {
            write!(f, "nullptr")
        } else {
            self.typename(f, config, TypeContext::Outbound)?;
            self.relation.decl(f, TypeContext::Outbound)?;
            write!(f, "{{}}")
        }
    }

    pub(crate) fn cast(
        &self,
        f: &mut fmt::Formatter<'_>,
        config: &AbiConfig,
        context: TypeContext,
    ) -> fmt::Result {
        if context != TypeContext::OutArg {
            write!(f, "(")?;
            if self.is_const {
                write!(f, "const ")?;
            }
            self.typename(f, config, context)?;
            if self.relation == TypeRelation::Ref || self.relation == TypeRelation::Owned && context == TypeContext::Outbound {
                TypeRelation::Ptr(1).decl(f, context)?;
            } else {
                self.relation.decl(f, context)?;
            }
            write!(f, ")")?;
        }
        Ok(())
    }

    /// Writes the underlying type name [Self::typ]. If `is_external` is set and this type is cast-mapped
    /// via [crate::AbiTypeConfig::cast_mapped], writes the mapped type instead.
    fn typename(
        &self,
        f: &mut fmt::Formatter<'_>,
        config: &AbiConfig,
        context: TypeContext,
    ) -> fmt::Result {
        let typ = config.types.resolved.get(&self.typ).unwrap_or(&self.typ);
        let ext = config.types.cast_mapped.get(typ).unwrap_or(typ);

        if context == TypeContext::Inbound {
            typ.cpp_case(f)
        } else if config.types.builtin.contains(ext) {
            ext.cpp_case(f)
        } else {
            ext.pascal_case(f)
        }
    }
}

/// The level of indirection between a variable's real type and its dereferenced type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TypeRelation {
    #[default]
    Value,
    Ref,
    Owned,
    Ptr(usize),
}

impl From<&query::FnArg::Match<'_>> for TypeRelation {
    fn from(value: &query::FnArg::Match<'_>) -> Self {
        if value.typ_refs.len() == 0 {
            TypeRelation::Value
        } else if value.typ_refs[0].r.as_str() == "&" {
            TypeRelation::Ref
        } else if value.typ_refs[0].r.as_str() == "&&" {
            TypeRelation::Owned
        } else {
            TypeRelation::Ptr(value.typ_refs.len())
        }
    }
}

impl From<&query::FnDef::Match<'_>> for TypeRelation {
    fn from(value: &query::FnDef::Match<'_>) -> Self {
        if value.ret_refs.len() == 0 {
            TypeRelation::Value
        } else if value.ret_refs[0].r.as_str() == "&" {
            TypeRelation::Ref
        } else if value.ret_refs[0].r.as_str() == "&&" {
            TypeRelation::Owned
        } else {
            TypeRelation::Ptr(value.ret_refs.len())
        }
    }
}

impl TypeRelation {
    pub(crate) fn nullcheck_depth(&self, context: TypeContext) -> usize {
        match (self, context) {
            (Self::Value, TypeContext::OutArg) => 1,
            (Self::Value, _) => 0,
            (Self::Ref, TypeContext::OutArg) => 1,
            (Self::Ref, _) => 1,
            (Self::Owned, TypeContext::OutArg) => 1,
            (Self::Owned, _) => 2,
            (Self::Ptr(_), TypeContext::OutArg) => 1,
            (Self::Ptr(_), _) => 0,
        }
    }

    pub(crate) fn decl(&self, f: &mut fmt::Formatter<'_>, context: TypeContext) -> fmt::Result {
        match (self, context) {
            (Self::Value, TypeContext::OutArg) => write!(f, "*"),
            (Self::Value, _) => Ok(()),
            (Self::Ref, TypeContext::OutArg) => write!(f, "**"),
            (Self::Ref, TypeContext::Outbound) => write!(f, "*"),
            (Self::Ref, TypeContext::Inbound) => write!(f, "&"),
            (Self::Owned, TypeContext::OutArg) => write!(f, "**"),
            (Self::Owned, TypeContext::Outbound) => write!(f, "**"),
            (Self::Owned, TypeContext::Inbound) => write!(f, "&&"),
            (Self::Ptr(depth), TypeContext::OutArg) => format::repeat(f, "*", depth + 1),
            (Self::Ptr(depth), _) => format::repeat(f, "*", *depth),
        }
    }

    /// The part of the deref that comes before the cast
    pub(crate) fn deref_pre_cast(
        &self,
        f: &mut fmt::Formatter<'_>,
        context: TypeContext,
    ) -> fmt::Result {
        match (self, context) {
            (_, TypeContext::OutArg) => write!(f, "*"),
            (Self::Ref, TypeContext::Inbound) => write!(f, "*"),
            _ => Ok(()),
        }
    }

    /// The part of the deref that comes after the cast
    pub(crate) fn deref_post_cast(
        &self,
        f: &mut fmt::Formatter<'_>,
        context: TypeContext,
    ) -> fmt::Result {
        match (self, context) {
            (Self::Ref, TypeContext::Outbound) => write!(f, "&"),
            (Self::Owned, TypeContext::Inbound) => write!(f, "*"),
            (Self::Owned, TypeContext::Outbound) => write!(f, "&"),
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod test {
    macro_rules! assert_decl (
        ($name:ident, [$typ:literal, $rel:ident, $context:ident], $expected:literal) => {
            assert_decl!($name, [$typ, $crate::TypeRelation::$rel, $context], $expected);
        };
        ($name:ident, [$typ:literal, Ptr($d:literal), $context:ident], $expected:literal) => {
            assert_decl!($name, [$typ, $crate::TypeRelation::Ptr($d), $context], $expected);
        };
        ($name:ident, [$typ:literal, $rel:expr, $context:ident], $expected:literal) => {
            #[test]
            fn $name() {
                assert_eq!(
                    std::fmt::from_fn(|f| {
                        crate::ApiType {
                            typ: $typ.into(),
                            is_const: false,
                            relation: $rel,
                        }.decl(f, &$crate::test::CONFIG, $crate::TypeContext::$context)?;
                        write!(f, " t")
                    }).to_string(),
                    $expected,
                );
            }
        };
    );

    mod decl {
        assert_decl!(value_inbound, ["a::b", Value, Inbound], "a::b t");
        assert_decl!(value_outbound, ["a::b", Value, Outbound], "AB t");
        assert_decl!(value_outarg, ["a::b", Value, OutArg], "AB* t");

        assert_decl!(ref_inbound, ["a::b", Ref, Inbound], "a::b& t");
        assert_decl!(ref_outbound, ["a::b", Ref, Outbound], "AB* t");
        assert_decl!(ref_outarg, ["a::b", Ref, OutArg], "AB** t");

        assert_decl!(owned_inbound, ["a::b", Owned, Inbound], "a::b&& t");
        assert_decl!(owned_outbound, ["a::b", Owned, Outbound], "AB** t");
        assert_decl!(owned_outarg, ["a::b", Owned, OutArg], "AB** t");

        assert_decl!(ptr1_inbound, ["a::b", Ptr(1), Inbound], "a::b* t");
        assert_decl!(ptr1_outbound, ["a::b", Ptr(1), Outbound], "AB* t");
        assert_decl!(ptr1_outarg, ["a::b", Ptr(1), OutArg], "AB** t");

        assert_decl!(ptr2_inbound, ["a::b", Ptr(2), Inbound], "a::b** t");
        assert_decl!(ptr2_outbound, ["a::b", Ptr(2), Outbound], "AB** t");
        assert_decl!(ptr2_outarg, ["a::b", Ptr(2), OutArg], "AB*** t");
    }

    macro_rules! assert_deref (
        ($name:ident, [$typ:literal, $rel:ident, $context:ident], $expected:literal) => {
            assert_deref!($name, [$typ, $crate::TypeRelation::$rel, $context], $expected);
        };
        ($name:ident, [$typ:literal, Ptr($d:literal), $context:ident], $expected:literal) => {
            assert_deref!($name, [$typ, $crate::TypeRelation::Ptr($d), $context], $expected);
        };
        ($name:ident, [$typ:literal, $rel:expr, $context:ident], $expected:literal) => {
            #[test]
            fn $name() {
                assert_eq!(
                    std::fmt::from_fn(|f| {
                        let t = crate::ApiType {
                            typ: $typ.into(),
                            is_const: false,
                            relation: $rel,
                        };
                        t.deref_pre_name(f, &$crate::test::CONFIG, $crate::TypeContext::$context)?;
                        write!(f, "t")?;
                        t.deref_post_name(f, &$crate::test::CONFIG, $crate::TypeContext::$context)
                    }).to_string(),
                    $expected,
                );
            }
        };
    );

    mod deref {
        assert_deref!(value_inbound, ["a::b", Value, Inbound], "(a::b)t");
        assert_deref!(value_outbound, ["a::b", Value, Outbound], "(AB)t");
        assert_deref!(value_outarg, ["a::b", Value, OutArg], "*t");

        assert_deref!(ref_inbound, ["a::b", Ref, Inbound], "*(a::b*)t");
        assert_deref!(ref_outbound, ["a::b", Ref, Outbound], "(AB*)&t");
        assert_deref!(ref_outarg, ["a::b", Ref, OutArg], "*t");

        assert_deref!(
            owned_inbound,
            ["a::b", Owned, Inbound],
            "std::move((a::b&&)*t)"
        );
        assert_deref!(
            owned_outbound,
            ["a::b", Owned, Outbound],
            "(AB*)&std::move(t)"
        );
        assert_deref!(owned_outarg, ["a::b", Owned, OutArg], "*t");

        assert_deref!(ptr1_inbound, ["a::b", Ptr(1), Inbound], "(a::b*)t");
        assert_deref!(ptr1_outbound, ["a::b", Ptr(1), Outbound], "(AB*)t");
        assert_deref!(ptr1_outarg, ["a::b", Ptr(1), OutArg], "*t");

        assert_deref!(ptr2_inbound, ["a::b", Ptr(2), Inbound], "(a::b**)t");
        assert_deref!(ptr2_outbound, ["a::b", Ptr(2), Outbound], "(AB**)t");
        assert_deref!(ptr2_outarg, ["a::b", Ptr(2), OutArg], "*t");
    }
}
