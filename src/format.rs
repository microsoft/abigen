mod node;
pub use node::*;

mod template;
pub use template::*;

use std::fmt::{self, Display, Write};

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

pub(crate) fn snake_case<T: AsRef<str>>(
    f: &mut fmt::Formatter<'_>,
    mut parts: impl Iterator<Item = T>,
) -> fmt::Result {
    if let Some(part) = parts.next() {
        write!(f, "{}", part.as_ref().to_lowercase())?;
    }
    while let Some(part) = parts.next() {
        write!(f, "_{}", part.as_ref().to_lowercase())?;
    }
    Ok(())
}

pub(crate) fn pascal_case<T: AsRef<str>>(mut parts: impl Iterator<Item = T>) -> String {
    let mut out = String::new();

    if let Some(part) = parts.next() {
        let _ = write!(out, "{}", part.as_ref());
    }
    while let Some(part) = parts.next() {
        let _ = write!(out, "_{}", part.as_ref().to_lowercase());
    }

    out
}

#[cfg(test)]
mod test {
    #[test]
    fn snake_case() {
        assert_eq!(
            fmt_wrap!(super::snake_case, ["x", "y", "MyClass"].iter()).to_string(),
            "x_y_myclass"
        );
    }
}
