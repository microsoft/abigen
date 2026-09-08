use std::{
    fmt::Display,
    fs, io,
    path::Path,
    str,
    sync::{LazyLock, PoisonError, RwLock, RwLockWriteGuard},
};

use tree_sitter::{LanguageError, Node, Parser, Tree, TreeCursor};

use crate::LANG;

static PARSER: LazyLock<RwLock<Parser>> = LazyLock::new(|| {
    let mut p = Parser::new();
    p.set_language(&LANG)
        .expect("Failed to set parser language");
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
    text: Vec<u8>,
    tree: Tree,
}

impl std::hash::Hash for Context {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.text.hash(state);
    }
}

impl Context {
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, ContextError> {
        let buf = fs::read(path)?;
        Self::from_buffer(buf)
    }

    pub fn from_buffer(buf: impl Into<Vec<u8>>) -> Result<Self, ContextError> {
        let buf = buf.into();

        let mut p = PARSER.write()?;
        let tree = p.parse(&buf, None).unwrap(); // cannot fail
        p.reset();

        Ok(Self { text: buf, tree })
    }

    pub fn root<'t>(&'t self) -> ContextNode<'t> {
        ContextNode {
            context: self,
            node: self.tree.root_node(),
        }
    }

    pub fn text<'t>(&'t self) -> &'t [u8] {
        &self.text as &[u8]
    }

    pub fn wrap<'t>(&'t self, node: Node<'t>) -> ContextNode<'t> {
        ContextNode {
            node,
            context: self,
        }
    }
}

#[derive(Clone, Copy, std::hash::Hash)]
pub struct ContextNode<'t> {
    pub context: &'t Context,
    pub node: Node<'t>,
}

impl<'t> ContextNode<'t> {
    pub fn walk(&'t self) -> ContextCursor<'t> {
        ContextCursor {
            context: self.context,
            cursor: self.node.walk(),
        }
    }

    pub fn as_str(&self) -> Result<&'t str, std::str::Utf8Error> {
        self.node.utf8_text(self.context.text())
    }
}

impl<'t> Into<Node<'t>> for ContextNode<'t> {
    fn into(self) -> Node<'t> {
        self.node
    }
}

impl<'a> std::fmt::Debug for ContextNode<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{}\"", self.as_str().map_err(|_| std::fmt::Error)?)
    }
}

impl PartialEq<ContextNode<'_>> for ContextNode<'_> {
    fn eq(&self, other: &ContextNode<'_>) -> bool {
        cursor_eq(&mut self.walk(), &mut other.walk())
    }
}

#[derive(Clone)]
pub struct ContextCursor<'t> {
    pub context: &'t Context,
    pub cursor: TreeCursor<'t>,
}

impl<'t> ContextCursor<'t> {
    pub fn node(&self) -> ContextNode<'t> {
        self.context.wrap(self.cursor.node())
    }
}

fn cursor_eq(a: &mut ContextCursor<'_>, b: &mut ContextCursor<'_>) -> bool {
    if a.cursor.node().kind_id() != b.cursor.node().kind_id() 
        || a.cursor.node().child_count() != b.cursor.node().child_count() {
        // a and b are superficially different different
        return false;
    } else if a.cursor.node().child_count() == 0 && a.node().as_str() != b.node().as_str() {
        // a and b are textually identical leaf nodes
        return true;
    }

    a.cursor.goto_first_child();
    b.cursor.goto_first_child();

    loop {
        if !cursor_eq(a, b) {
            return false;
        } else if !a.cursor.goto_next_sibling() || !b.cursor.goto_next_sibling() {
            break;
        }
    }

    a.cursor.goto_parent();
    b.cursor.goto_parent();

    true
}