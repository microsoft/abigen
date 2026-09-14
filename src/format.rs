mod node;
pub use node::*;

mod template;
pub use template::*;

mod fn_def;

use std::fmt::Display;

pub(crate) struct Delimited<const D: char, T> where T: Display {
    items: Vec<T>,
}

impl<const D: char, I> FromIterator<I> for Delimited<D, I> where I: Display {
    fn from_iter<T: IntoIterator<Item = I>>(iter: T) -> Self {
        Self {
            items: iter.into_iter().collect(),
        }
    }
}
impl<const D: char, T> Display for Delimited<D, T> where T: Display {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, x) in self.items.iter().enumerate() {
            if i > 0 {
                write!(f, "{D}")?;
            }
            write!(f, "{x}")?;
        }
        Ok(())
    }
}

pub(crate) fn indent(i: u32) -> impl std::fmt::Display {
    std::fmt::from_fn(move |f| {
        for _ in 0..i {
            write!(f, "  ")?
        }
        Ok(())
    })
}