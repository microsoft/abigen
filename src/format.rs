mod node;
pub use node::*;

mod template;
pub use template::*;

use std::fmt::{self, Display};

macro_rules! fmt_wrap(
    ($self:ident.$fn:ident $(, $($arg:expr),*)? $(,)?) => {
        std::fmt::from_fn(move |f| $self.$fn(f $(, $($arg),*)?))
    };
    ($fn:expr $(, $($arg:expr),*)? $(,)?) => {
        std::fmt::from_fn(|f| $fn(f $(, $($arg),*)?))
    };
);
pub(crate) use fmt_wrap;

pub(crate) fn delimited<T: Display>(
    f: &mut fmt::Formatter<'_>,
    d: &str,
    mut parts: impl Iterator<Item = T>,
) -> fmt::Result {
    if let Some(part) = parts.next() {
        write!(f, "{part}")?;
    }
    while let Some(part) = parts.next() {
        write!(f, "{d}{part}")?;
    }
    Ok(())
}

pub(crate) fn repeat(f: &mut fmt::Formatter<'_>, s: &str, i: usize) -> fmt::Result {
    for _ in 0..i {
        write!(f, "{s}")?
    }
    Ok(())
}
