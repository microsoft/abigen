use std::{
    fmt::{self, Debug, Display},
    fs,
    path::Path,
    str,
    sync::{LazyLock, RwLock},
};

use tree_sitter::{Node, Parser, Tree, TreeCursor};

use crate::{Error, LANG};

static PARSER: LazyLock<RwLock<Parser>> = LazyLock::new(|| {
    let mut p = Parser::new();
    p.set_language(&LANG)
        .expect("Failed to set parser language");
    RwLock::new(p)
});

#[derive(Debug, Clone)]
pub struct Context {
    text: String,
    tree: Tree,
}

impl std::hash::Hash for Context {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.text.hash(state);
    }
}

impl Context {
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, Error> {
        let buf = fs::read(path)?;
        Self::from_buffer(buf)
    }

    pub fn from_buffer(buf: impl Into<Vec<u8>>) -> Result<Self, Error> {
        let buf = String::from_utf8(buf.into())?;

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

    pub fn wrap<'t>(&'t self, node: Node<'t>) -> ContextNode<'t> {
        ContextNode {
            node,
            context: self,
        }
    }

    pub fn as_str(&self) -> &str {
        self.text.as_str()
    }

    pub fn into_string(self) -> String {
        self.text
    }
}

impl From<String> for Context {
    fn from(value: String) -> Self {
        let mut p = PARSER.write().expect("Parser poisoned!");
        let tree = p.parse(&value, None).unwrap(); // cannot fail
        p.reset();

        Self { text: value, tree }
    }
}

impl AsRef<str> for Context {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::fmt::Display for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Clone, Copy, std::hash::Hash)]
pub struct ContextNode<'t> {
    pub context: &'t Context,
    pub node: Node<'t>,
}

impl<'t> ContextNode<'t> {
    pub fn walk(&self) -> ContextCursor<'t> {
        ContextCursor {
            context: self.context,
            cursor: self.node.walk(),
        }
    }

    pub fn as_str(&self) -> &'t str {
        &self.context.as_str()[self.node.start_byte()..self.node.end_byte()]
    }

    pub fn diff<'b>(&self, other: &ContextNode<'b>) -> Result<(), EqError<'t, 'b>> {
        cursor_eq(&mut self.walk(), &mut other.walk())
    }
}

impl<'t> Into<Node<'t>> for ContextNode<'t> {
    fn into(self) -> Node<'t> {
        self.node
    }
}

impl<'t> AsRef<str> for ContextNode<'t> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<'a> std::fmt::Debug for ContextNode<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{}\"", self.as_str().escape_debug())
    }
}

impl PartialEq<ContextNode<'_>> for ContextNode<'_> {
    fn eq(&self, other: &ContextNode<'_>) -> bool {
        cursor_eq(&mut self.walk(), &mut other.walk()).is_ok()
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

#[derive(Clone, Copy)]
pub struct EqError<'a, 'e> {
    pub actual: ContextNode<'a>,
    pub expected: ContextNode<'e>,
}
impl<'a, 'e> EqError<'a, 'e> {
    pub fn new(a: ContextNode<'a>, e: ContextNode<'e>) -> Self {
        Self {
            actual: a,
            expected: e,
        }
    }
}
impl Debug for EqError<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Expected at {}:\n{}",
            self.expected.node.start_position(),
            self.expected.as_str()
        )?;
        write!(
            f,
            "Actual at {}:\n{}",
            self.actual.node.start_position(),
            self.actual.as_str()
        )
    }
}
impl Display for EqError<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl std::error::Error for EqError<'_, '_> {}

fn cursor_eq<'a, 'e>(
    a: &mut ContextCursor<'a>,
    e: &mut ContextCursor<'e>,
) -> Result<(), EqError<'a, 'e>> {
    if a.cursor.node().kind_id() != e.cursor.node().kind_id()
        || a.cursor.node().child_count() != e.cursor.node().child_count()
    {
        // a and b are superficially different
        return Err(EqError::new(a.node(), e.node()));
    }

    if a.cursor.node().child_count() == 0 {
        if a.node().as_str() == e.node().as_str() {
            // a and b are textually identical leaf nodes
            return Ok(());
        } else {
            return Err(EqError::new(a.node(), e.node()));
        }
    }

    a.cursor.goto_first_child();
    e.cursor.goto_first_child();

    loop {
        if let Err(e) = cursor_eq(a, e) {
            return Err(e);
        } else if !a.cursor.goto_next_sibling() || !e.cursor.goto_next_sibling() {
            break;
        }
    }

    a.cursor.goto_parent();
    e.cursor.goto_parent();

    Ok(())
}
