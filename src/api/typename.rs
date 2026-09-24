use crate::{
    format,
    query::{f, k},
};
use std::{
    fmt::{self, Display},
    sync::LazyLock,
};

static VOID_T: LazyLock<TypeName> = LazyLock::new(|| TypeName::from("void"));

#[derive(Debug, Clone, Copy)]
pub struct TypeParseError {
    pub kind: &'static str,
}

impl TypeParseError {
    pub fn new(kind: &'static str) -> Self {
        Self { kind }
    }
}

impl Display for TypeParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Failed to parse type from node. Expected an identifier, got '{}'",
            self.kind
        )
    }
}
impl std::error::Error for TypeParseError {}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    std::hash::Hash,
    serde::Serialize,
    serde::Deserialize,
)]
#[serde(from = "&str")]
pub struct TypeName {
    parts: Vec<String>,
}

impl<T: AsRef<str>> From<T> for TypeName {
    fn from(value: T) -> Self {
        Self {
            parts: value
                .as_ref()
                .split("::")
                .map(|p| String::from(p))
                .collect(),
        }
    }
}

impl<T: Into<String>> FromIterator<T> for TypeName {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self {
            parts: iter.into_iter().map(|i| i.into()).collect(),
        }
    }
}

impl IntoIterator for TypeName {
    type Item = String;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.parts.into_iter()
    }
}

impl Default for TypeName {
    fn default() -> Self {
        VOID_T.clone()
    }
}

impl TypeName {
    pub fn void() -> &'static Self {
        &VOID_T
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_node(node: ContextNode<'_>) -> Result<Self, TypeParseError> {
        fn collect(name: ContextNode<'_>, out: &mut Vec<String>) -> Result<(), TypeParseError> {
            if name.node.kind_id() == *k::TYPE_ID {
                out.push(name.as_str().to_string());
                Ok(())
            } else if name.node.kind_id() == *k::QUALIFIED_ID {
                let scope = name
                    .child_by_field_id(*f::SCOPE)
                    .ok_or(TypeParseError::new(name.node.kind()))?;
                let name = name
                    .child_by_field_id(*f::NAME)
                    .ok_or(TypeParseError::new(name.node.kind()))?;

                if scope.node.kind_id() == *k::NS_ID {
                    out.push(scope.as_str().to_string());
                } else {
                    return Err(TypeParseError::new(scope.node.kind()));
                }

                collect(name, out)
            } else {
                Err(TypeParseError::new(name.node.kind()))
            }
        }

        let mut parts = vec![];
        collect(node, &mut parts)?;
        Ok(Self { parts })
    }

    pub fn push(&mut self, part: impl Into<String>) {
        self.parts.push(part.into());
    }

    pub fn insert(&mut self, index: usize, part: impl Into<String>) {
        self.parts.insert(index, part.into());
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = &str> {
        self.parts.iter().map(|s| s.as_str())
    }

    pub fn len(&self) -> usize {
        self.parts.len()
    }

    pub fn cpp_case(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        format::delimited(f, "::", self.parts.iter())
    }

    pub fn snake_case(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = self.parts.iter();
        if let Some(part) = parts.next() {
            write!(f, "{}", part.to_lowercase())?;
        }
        while let Some(part) = parts.next() {
            write!(f, "_{}", part.to_lowercase())?;
        }
        Ok(())
    }

    pub fn pascal_case(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for part in self.parts.iter() {
            for sub in part.split('_') {
                if let Some(c) = sub.chars().next() {
                    write!(f, "{}", c.to_uppercase())?;
                }
                if sub.len() > 1 {
                    write!(f, "{}", &sub[1..])?
                }
            }
        }
        Ok(())
    }
}

macro_rules! name_concat(
    ($first:expr $(, $part:expr)* $(,)?) => {
        $first.into_iter().map(|p| String::from(p))
        .$(chain($part.into_iter().map(|p| String::from(p)))).+
        .collect::<$crate::TypeName>()
    };
);
pub(crate) use name_concat;

use crate::ContextNode;

#[cfg(test)]
mod test {
    use super::*;
    use crate::format::fmt_wrap;

    #[test]
    fn concat() {
        assert_eq!(
            TypeName::from_iter(["a", "b", "c", "d"]),
            name_concat!(
                ["a"],
                TypeName::from_iter(["b", "c"]).iter(),
                TypeName::from_iter(["d"])
            ),
        );
    }

    #[test]
    fn snake_case() {
        let typ = TypeName::from_iter(["x", "y", "MyClass"]);
        assert_eq!(fmt_wrap!(typ.snake_case).to_string(), "x_y_myclass");
    }
}
