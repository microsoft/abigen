mod node;
pub use node::*;

mod template;
pub use template::*;

use std::fmt::{self, Display};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Delimited<const D: char> {
    pub parts: Vec<String>,
}

impl<const D: char, I> FromIterator<I> for Delimited<D>
where
    I: Display,
{
    fn from_iter<T: IntoIterator<Item = I>>(iter: T) -> Self {
        Self {
            parts: iter.into_iter().map(|i| i.to_string()).collect(),
        }
    }
}

impl<const D: char> Display for Delimited<D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut iter = self.parts.iter();
        if let Some(p) = iter.next() {
            write!(f, "{p}")?;
        }
        while let Some(p) = iter.next() {
            write!(f, "{D}{p}")?;
        }
        Ok(())
    }
}

pub(crate) fn indent(i: u32) -> impl std::fmt::Display {
    fmt::from_fn(move |f| {
        for _ in 0..i {
            write!(f, "  ")?
        }
        Ok(())
    })
}
