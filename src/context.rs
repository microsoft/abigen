use std::{fs, io, path::Path, str, fmt::Display};

use tree_sitter::{LanguageError, Node, Parser, Query, QueryCursor, QueryMatches, Tree, TreeCursor};

use crate::LANG;

#[derive(Debug)]
pub enum ContextError {
    Language(LanguageError),
    Io(io::Error),
    Utf8(str::Utf8Error),
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

impl Display for ContextError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Language(e) => write!(f, "{e}"),
            Self::Io(e) => write!(f, "{e}"),
            Self::Utf8(e) => write!(f, "{e}"),
        }
    }
}
impl std::error::Error for ContextError {}

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
        let mut parser = Parser::new();
        parser.set_language(&LANG)?;

        let buf = buf.into();
        let tree = parser.parse(&buf, None).unwrap();
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
}

pub struct InContext<'t, T> {
    pub context: &'t Context,
    pub item: T,
}

impl<'t, T> InContext<'t, T> {
    pub fn new(item: T, context: &'t Context) -> Self {
        Self { context, item }
    }
}

impl<'t> InContext<'t, Node<'t>> {
    pub fn text(&self) -> Result<&str, ContextError> {
        self.item.utf8_text(&self.context.text).map_err(ContextError::from)
    }

    pub fn text_trunc(&self, max_len: usize) -> Result<&str, ContextError> {
        if self.item.byte_range().len() > max_len {
            str::from_utf8(
                &self.context.text[self.item.start_byte() .. self.item.start_byte() + max_len],
            ).map_err(ContextError::from)
        } else {
            self.text()
        }
    }

    pub fn matches<'q, 'c>(&self, query: &'q Query, cursor: &'c mut QueryCursor) -> QueryMatches<'q, 't, &[u8], &[u8]>
    where 'c: 'q {
        cursor.matches(query, self.item, &self.context.text as &[u8])
    }

    pub fn walk(&'t self) -> InContext<'t, TreeCursor<'t>> {
        InContext::new(self.item.walk(), self.context)
    }
}

impl<'t> InContext<'t, TreeCursor<'t>> {
    pub fn node(&self) -> InContext<'t, Node<'t>> {
        InContext::new(self.item.node(), self.context)
    }
}
