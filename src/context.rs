use std::{
    borrow::Cow, fs, path::Path, str, sync::{LazyLock, RwLock},
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

        Ok(Self {
            text: buf,
            tree,
        })
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

        Self {
            text: value,
            tree,
        }
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
    pub fn walk(&'t self) -> ContextCursor<'t> {
        ContextCursor {
            context: self.context,
            cursor: self.node.walk(),
        }
    }

    pub fn as_str(&self) -> &'t str {
        &self.context.as_str()[self.node.start_byte()..self.node.end_byte()]
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