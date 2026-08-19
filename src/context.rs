use std::{fmt::Display, fs, io, path::Path, str, sync::{LazyLock, RwLock, RwLockWriteGuard, PoisonError}};

use tree_sitter::{LanguageError, Node, Parser, Tree, TreeCursor};


use crate::LANG;

static PARSER: LazyLock<RwLock<Parser>> = LazyLock::new(|| {
    let mut p = Parser::new();
    p.set_language(&LANG).expect("Failed to set parser language");
    RwLock::new(p)
});

#[derive(Debug)]
pub enum ContextError {
    Language(LanguageError),
    Io(io::Error),
    Utf8(str::Utf8Error),
    Poison(PoisonError<RwLockWriteGuard<'static, Parser>>),
}
impl From<LanguageError> for ContextError {
    fn from(value: LanguageError) -> Self {
        Self::Language(value)
    }
}
impl From<io::Error> for ContextError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}
impl From<str::Utf8Error> for ContextError {
    fn from(value: str::Utf8Error) -> Self {
        Self::Utf8(value)
    }
}
impl From<PoisonError<RwLockWriteGuard<'static, Parser>>> for ContextError {
    fn from(value: PoisonError<RwLockWriteGuard<'static, Parser>>) -> Self {
        Self::Poison(value)
    }
}
impl Into<std::fmt::Error> for ContextError {
    fn into(self) -> std::fmt::Error {
        std::fmt::Error
    }
}

impl Display for ContextError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Language(e) => write!(f, "{e}"),
            Self::Io(e) => write!(f, "{e}"),
            Self::Utf8(e) => write!(f, "{e}"),
            Self::Poison(e) => write!(f, "{e}"),
        }
    }
}
impl std::error::Error for ContextError {}

#[derive(Debug, Clone)]
pub struct Context {
    pub text: Vec<u8>,
    tree: Tree,
}

impl Context {
    pub fn try_from_file(path: &Path) -> Result<Self, ContextError> {
        let buf = fs::read(path)?;
        Self::try_from_buffer(buf)
    }

    pub fn try_from_buffer(buf: impl Into<Vec<u8>>) -> Result<Self, ContextError> {
        let buf = buf.into();
        
        let mut p = PARSER.write()?;
        let tree = p.parse(&buf, None).unwrap(); // cannot fail
        p.reset();

        Ok(Self {
            text: buf,
            tree,
        })
    }

    pub fn root<'t>(&'t self) -> InContext<'t, Node<'t>> {
        InContext {
            context: self,
            item: self.tree.root_node(),
        }
    }

    pub fn text<'t>(&'t self) -> &'t [u8] {
        &self.text as &[u8]
    }

    pub fn wrap<T>(&self, item: T) -> InContext<'_, T> {
        InContext { item, context: self }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct InContext<'t, T> {
    pub context: &'t Context,
    pub item: T,
}

impl<'t> InContext<'t, Node<'t>> {
    pub fn walk(&'t self) -> InContext<'t, TreeCursor<'t>> {
        self.context.wrap(self.item.walk())
    }
}
impl<'t> Into<Node<'t>> for InContext<'t, Node<'t>> {
    fn into(self) -> Node<'t> {
        self.item
    }
}

impl<'t> InContext<'t, TreeCursor<'t>> {
    pub fn node(&self) -> InContext<'t, Node<'t>> {
        self.context.wrap(self.item.node())
    }
}
