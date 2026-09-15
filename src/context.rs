use std::{
    fmt::{self, Debug, Display},
    fs,
    path::Path,
    str,
    sync::{LazyLock, RwLock},
};

use tree_sitter::{Node, Parser, Point, Range, Tree, TreeCursor};

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

    pub fn children(&self, cursor: &mut ContextCursor<'t>) -> impl ExactSizeIterator<Item=Self> {
        self.node.children(&mut cursor.cursor).map(|c| self.context.wrap(c))
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
    pub actual: &'a Context,
    pub actual_range: Range,
    pub expected: &'e Context,
    pub expected_range: Range,
}
impl Debug for EqError<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Expected at {}:\n{}",
            self.expected_range.start_point,
            &self.expected.as_str()[self.expected_range.start_byte..self.expected_range.end_byte],
        )?;
        write!(
            f,
            "Actual at {}:\n{}",
            self.actual_range.start_point,
            &self.actual.as_str()[self.actual_range.start_byte..self.actual_range.end_byte],
        )
    }
}
impl Display for EqError<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl std::error::Error for EqError<'_, '_> {}

impl<'a, 'e> EqError<'a, 'e> {
    pub fn new(a: ContextNode<'a>, e: ContextNode<'e>) -> Self {
        Self {
            actual: a.context,
            actual_range: a.node.range(),
            expected: e.context,
            expected_range: e.node.range(),
        }
    }

    pub fn missing(a_ctx: &'a Context, a_byte: usize, a_point: Point, e: ContextNode<'e>) -> Self {
        Self {
            actual: a_ctx,
            actual_range: Range {
                start_byte: a_byte,
                start_point: a_point,
                end_byte: a_byte,
                end_point: a_point,
            },
            expected: e.context,
            expected_range: e.node.range(),
        }
    }

    pub fn extra(a: ContextNode<'a>, e_ctx: &'e Context, e_byte: usize, e_point: Point) -> Self {
        Self {
            actual: a.context,
            actual_range: a.node.range(),
            expected: e_ctx,
            expected_range: Range {
                start_byte: e_byte,
                start_point: e_point,
                end_byte: e_byte,
                end_point: e_point,
            },
        }
    }
}

impl<'a, 'e> std::ops::Add for EqError<'a, 'e> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self::Output {
            actual: self.actual,
            actual_range: Range {
                start_byte: self.actual_range.start_byte.min(rhs.actual_range.start_byte),
                start_point: self.actual_range.start_point.min(rhs.actual_range.start_point),
                end_byte: self.actual_range.end_byte.max(rhs.actual_range.end_byte),
                end_point: self.actual_range.end_point.max(rhs.actual_range.end_point),
            },
            expected: self.expected,
            expected_range: Range {
                start_byte: self.expected_range.start_byte.min(rhs.expected_range.start_byte),
                start_point: self.expected_range.start_point.min(rhs.expected_range.start_point),
                end_byte: self.expected_range.end_byte.max(rhs.expected_range.end_byte),
                end_point: self.expected_range.end_point.max(rhs.expected_range.end_point),
            },
        }
    }
}

fn cursor_eq<'a, 'e>(
    a: &mut ContextCursor<'a>,
    e: &mut ContextCursor<'e>,
) -> Result<(), EqError<'a, 'e>> {
    if a.cursor.node().kind_id() != e.cursor.node().kind_id()
    {
        // a and b are superficially different
        return Err(EqError::new(a.node(), e.node()));
    }

    if a.cursor.node().child_count() == 0 && e.cursor.node().child_count() == 0 {
        if a.node().as_str().trim() == e.node().as_str().trim() {
            // a and b are textually identical leaf nodes
            return Ok(());
        } else {
            return Err(EqError::new(a.node(), e.node()));
        }
    }

    let mut error = None;
    
    let a_has_children = a.cursor.goto_first_child();
    let e_has_children = e.cursor.goto_first_child();
    if a_has_children && !e_has_children {
        let err = EqError::extra(
            a.node(), 
            e.context, 
            e.node().node.end_byte(), 
            e.node().node.end_position());
        error = error.map_or(Some(err), |old| Some(old + err));

        while a.cursor.goto_next_sibling() {
            let err = EqError::extra(
                a.node(), 
                e.context, 
                e.node().node.end_byte(), 
                e.node().node.end_position(),
            );
            error = error.map_or(Some(err), |old| Some(old + err));
        }

        return Err(error.unwrap());
    } else if e_has_children && !a_has_children {
        let err = EqError::missing(
            a.context, 
            a.node().node.end_byte(), 
            a.node().node.end_position(), 
            e.node());
        error = error.map_or(Some(err), |old| Some(old + err));

        while a.cursor.goto_next_sibling() {
            let err = EqError::missing(
                a.context, 
                a.node().node.end_byte(), 
                a.node().node.end_position(), 
                e.node());
            error = error.map_or(Some(err), |old| Some(old + err));
        }

        return Err(error.unwrap());
    }

    loop {
        if let Err(e) = cursor_eq(a, e) {
            error = error.map_or(Some(e), |old| Some(old + e));
            break;
        }
        
        if !e.cursor.goto_next_sibling() {
            while a.cursor.goto_next_sibling() {
                let e = EqError::extra(
                    a.node(), 
                    e.context, 
                    e.node().node.end_byte(), 
                    e.node().node.end_position(),
                );
                error = error.map_or(Some(e), |old| Some(old + e));
            }
            break;
        } else if !a.cursor.goto_next_sibling() {
            while e.cursor.goto_next_sibling() {
                let e = EqError::missing(
                    a.context, 
                    a.node().node.end_byte(), 
                    a.node().node.end_position(), 
                    e.node(),
                );
                error = error.map_or(Some(e), |old| Some(old + e));
            }
            break;
        }
    }

    a.cursor.goto_parent();
    e.cursor.goto_parent();

    if error.is_some() {
        a.cursor.goto_last_child();
        e.cursor.goto_last_child();
        
        loop {
            if let Err(e) = cursor_eq(a, e) {
            error = error.map_or(Some(e), |old| Some(old + e));
            break;
        }
        
        if !e.cursor.goto_previous_sibling() {
            while a.cursor.goto_previous_sibling() {
                let e = EqError::extra(
                    a.node(), 
                    e.context, 
                    e.node().node.end_byte(), 
                    e.node().node.end_position(),
                );
                error = error.map_or(Some(e), |old| Some(old + e));
            }
            break;
        } else if !a.cursor.goto_previous_sibling() {
            while e.cursor.goto_previous_sibling() {
                let e = EqError::missing(
                    a.context, 
                    a.node().node.end_byte(), 
                    a.node().node.end_position(), 
                    e.node(),
                );
                error = error.map_or(Some(e), |old| Some(old + e));
            }
            break;
        }
        }

        a.cursor.goto_parent();
        e.cursor.goto_parent();
    }

    error.map(|e| Err(e)).unwrap_or(Ok(()))
}
