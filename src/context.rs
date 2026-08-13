use std::{fs, io, path::Path, str, fmt::Display};

use tree_sitter::{LanguageError, Node, Parser, Query, QueryCursor, QueryError, QueryMatches, StreamingIterator, Tree};

use crate::LANG;

#[derive(Debug)]
pub enum ContextError {
    Language(LanguageError),
    Io(io::Error),
    Utf8(str::Utf8Error),
    Query(QueryError),
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
impl From<QueryError> for ContextError {
    fn from(value: QueryError) -> Self {
        Self::Query(value)
    }
}

impl Display for ContextError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Language(e) => write!(f, "{e}"),
            Self::Io(e) => write!(f, "{e}"),
            Self::Utf8(e) => write!(f, "{e}"),
            Self::Query(e) => write!(f, "{e}"),
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

    pub fn root<'a>(&'a self) -> InContext<'a, Node> {
        InContext {
            context: self,
            item: self.tree.root_node(),
        }
    }
}

pub struct InContext<'a, T> {
    pub context: &'a Context,
    pub item: T,
}

impl<'a> InContext<'a, Node<'a>> {
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

    pub fn matches(self, query: &str) -> Result<QueryContext<'a>, ContextError> {
        QueryContext::new(self, query)
    }
}

pub struct QueryContext<'t> {
    pub node: InContext<'t, Node<'t>>,
    pub query: Query,
    pub cursor: QueryCursor,
}
impl<'t> QueryContext<'t> {
    pub fn new(node: InContext<'t, Node<'t>>, query: &str) -> Result<Self, ContextError> {
        let query = Query::new(&LANG, query)?;
        let mut cursor = QueryCursor::new();
        Ok(Self {
            node, query, cursor
        })
    }

    pub fn matches<'q>(&'q self) -> InQueryContext<'q, 't, QueryMatches<'q, 't, &'t [u8], &'t [u8]>> {
        let m = self.cursor.matches(
            &self.query, 
            self.node.item, 
            &self.node.context.text as &[u8],
        );
        InQueryContext {
            context: self,
            item: m,
        }
    }
}

pub struct InQueryContext<'q, 't, T> {
    pub context: &'q QueryContext<'t>,
    pub item: T,
}