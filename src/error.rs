use tree_sitter::{LanguageError, Parser};
use std::{io, path::PathBuf, str, sync::{PoisonError, RwLockWriteGuard}};

#[derive(Debug)]
pub enum Error {
    Language(LanguageError),
    Io(io::Error),
    Utf8(str::Utf8Error),
    Poison(PoisonError<RwLockWriteGuard<'static, Parser>>),
    Template { file: PathBuf, message: String },
}
impl From<LanguageError> for Error {
    fn from(value: LanguageError) -> Self {
        Self::Language(value)
    }
}
impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}
impl From<str::Utf8Error> for Error {
    fn from(value: str::Utf8Error) -> Self {
        Self::Utf8(value)
    }
}
impl From<PoisonError<RwLockWriteGuard<'static, Parser>>> for Error {
    fn from(value: PoisonError<RwLockWriteGuard<'static, Parser>>) -> Self {
        Self::Poison(value)
    }
}
impl Into<std::fmt::Error> for Error {
    fn into(self) -> std::fmt::Error {
        std::fmt::Error
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Language(e) => write!(f, "{e}"),
            Self::Io(e) => write!(f, "{e}"),
            Self::Utf8(e) => write!(f, "{e}"),
            Self::Poison(e) => write!(f, "{e}"),
            Self::Template { file, message } => {
                write!(f, "Error processing template file '{file}': {message}", file = file.display())
            },
        }
    }
}
impl std::error::Error for Error {}
