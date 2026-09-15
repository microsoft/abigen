mod node;
pub use node::*;

mod template;
pub use template::*;

use std::fmt::{self, Display, Write};

pub(crate) struct Delimited<const D: char>(String);

impl<const D: char, I> FromIterator<I> for Delimited<D>
where
    I: Display,
{
    fn from_iter<T: IntoIterator<Item = I>>(iter: T) -> Self {
        let mut s = String::new();
        let mut iter = iter.into_iter();
        if let Some(i) = iter.next() {
            write!(s, "{i}").unwrap();
        }
        while let Some(i) = iter.next() {
            write!(s, "{D}{i}").unwrap();
        }
        Self(s)
    }
}

impl<const D: char> Display for Delimited<D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<const D: char> Into<String> for Delimited<D> {
    fn into(self) -> String {
        self.0
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
