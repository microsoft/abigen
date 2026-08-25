use std::{collections::HashMap, fmt::Display, str::Utf8Error, sync::LazyLock};
use tree_sitter::{Node, TreeCursor};
use crate::InContext;

const INDENT: &str = "  ";
static STRIP_MAP: LazyLock<HashMap<char, char>> = LazyLock::new(|| {
    HashMap::from_iter([
        ('\n', '␤'),
        ('\r', '␍'),
    ])
});

fn sanitize(item: char) -> char {
    if let Some(clean) = STRIP_MAP.get(&item) {
        *clean
    } else {
        item
    }
}

pub fn print_node(node: InContext<Node>, max_depth: u32, max_code: usize) {
    print_tree(node.walk(), max_depth, max_code);
}

pub fn print_tree<'a>(mut cursor: InContext<'a, TreeCursor<'a>>, max_depth: u32, max_code: usize) -> InContext<'a, TreeCursor<'a>> {
    if !cursor.item.node().is_named() {
        return cursor;
    }

    for _ in 0..cursor.item.depth() {
        print!("{INDENT}");
    }

    println!("{}",
        NodeFormat {
            node: cursor.node(),
            field: cursor.item.field_name(),
            max_code,
        },
    );

    if cursor.item.depth() == max_depth - 1 && cursor.node().item.named_child_count() > 0 {
        for _ in 0 .. max_depth {
            print!("{INDENT}");
        }
        println!("+{} children", cursor.node().item.named_child_count());
    } else if cursor.item.goto_first_child() {
        cursor = print_tree(cursor, max_depth, max_code);
        while cursor.item.goto_next_sibling() {
            cursor = print_tree(cursor, max_depth, max_code);
        }
        cursor.item.goto_parent();
    }

    cursor
}

pub struct NodeFormat<'a> {
    pub node: InContext<'a, Node<'a>>, 
    pub field: Option<&'static str>,
    pub max_code: usize,
}

impl Display for NodeFormat<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let body: String = self.node.as_str().map_err(|_| std::fmt::Error)?
            .chars().map(sanitize).collect();
        let body = if body.len() > self.max_code {
            format!("\"{}...\" +{} chars",
                &body[0 .. self.max_code],
                body.len() - self.max_code,
            )
        } else {
            format!("\"{}\"", body)
        };

        match self.field {
            Some(field) => {
                write!(f, "({field}) {typ} {body}", typ = self.node.item.kind())
            },
            None => {
                write!(f, "{typ} {body}", typ = self.node.item.kind())
            },
        }
    }
}

impl<'a> InContext<'a, Node<'a>> {
    pub fn as_str(&self) -> Result<&'a str, Utf8Error> {
        self.item.utf8_text(&self.context.text)
    }
}

impl<'a> Display for InContext<'a, Node<'a>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str().map_err(|_| std::fmt::Error)?)
    }
}

impl<'a> AsRef<str> for InContext<'a, Node<'a>> {
    fn as_ref(&self) -> &str {
        self.as_str().expect("Utf8 error")
    }
}