use std::fmt;

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, std::hash::Hash)]
pub struct TypeName {
    parts: Vec<String>,
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

impl TypeName {
    pub fn new() -> Self {
        Self::default()
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
    ($first:expr, $($part:expr),* $(,)?) => {
        $first.into_iter().map(|p| String::from(p))
        .$(chain($part.into_iter().map(|p| String::from(p)))).+
        .collect::<$crate::api::typename::TypeName>()
    };
);
pub(crate) use name_concat;

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
