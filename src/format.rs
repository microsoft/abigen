use crate::{ContextCursor, ContextNode};
use std::{collections::HashMap, fmt::Display, sync::LazyLock};

const INDENT: &str = "  ";
static STRIP_MAP: LazyLock<HashMap<char, char>> =
    LazyLock::new(|| HashMap::from_iter([('\n', '␤'), ('\r', '␍')]));

fn sanitize(item: char) -> char {
    if let Some(clean) = STRIP_MAP.get(&item) {
        *clean
    } else {
        item
    }
}

pub fn print_node(node: ContextNode, max_depth: u32, max_code: usize, show_anon: bool) {
    print_tree(node.walk(), max_depth, max_code, show_anon);
}

pub fn print_tree<'a>(
    mut cursor: ContextCursor<'a>,
    max_depth: u32,
    max_code: usize,
    show_anon: bool,
) -> ContextCursor<'a> {
    if !show_anon && !cursor.cursor.node().is_named() {
        return cursor;
    }

    for _ in 0..cursor.cursor.depth() {
        print!("{INDENT}");
    }

    println!(
        "{}",
        NodeFormat {
            node: cursor.node(),
            field: cursor.cursor.field_name(),
            max_code,
        },
    );

    if cursor.cursor.depth() == max_depth - 1 && cursor.node().node.named_child_count() > 0 {
        for _ in 0..max_depth {
            print!("{INDENT}");
        }
        println!("+{} children", cursor.node().node.named_child_count());
    } else if cursor.cursor.goto_first_child() {
        cursor = print_tree(cursor, max_depth, max_code, show_anon);
        while cursor.cursor.goto_next_sibling() {
            cursor = print_tree(cursor, max_depth, max_code, show_anon);
        }
        cursor.cursor.goto_parent();
    }

    cursor
}

pub struct NodeFormat<'a> {
    pub node: ContextNode<'a>,
    pub field: Option<&'static str>,
    pub max_code: usize,
}

impl Display for NodeFormat<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let body: String = self
            .node
            .as_str()
            .map_err(|_| std::fmt::Error)?
            .chars()
            .map(sanitize)
            .collect();
        let body = if body.len() > self.max_code {
            format!(
                "\"{}...\" +{} chars",
                &body[0..self.max_code],
                body.len() - self.max_code,
            )
        } else {
            format!("\"{}\"", body)
        };

        match self.field {
            Some(field) if self.node.node.is_named() => {
                write!(f, "({field}) {typ} {body}", typ = self.node.node.kind())
            }
            Some(field) => {
                write!(f, "({field}) -{typ} {body}", typ = self.node.node.kind())
            }
            None if self.node.node.is_named() => {
                write!(f, "{typ} {body}", typ = self.node.node.kind())
            }
            None => {
                write!(f, "-{typ} {body}", typ = self.node.node.kind())
            }
        }
    }
}
