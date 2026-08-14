use std::fmt::Display;
use tree_sitter::{Node, TreeCursor};
use crate::InContext;

const INDENT: &str = "  ";

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
        let node = self.node.item;
        let body = if node.byte_range().len() > self.max_code {
            format!("\"{}...\" +{} bytes",
                str::from_utf8(&self.node.context.text[node.start_byte() .. node.start_byte() + self.max_code])
                    .map_err(|_| std::fmt::Error)?
                    .replace('\n', "\u{2424}"),
                node.byte_range().len() - self.max_code,
            )
        } else {
            format!("\"{}\"",
                self.node.text()
                    .map_err(|_| std::fmt::Error)?
                    .replace('\n', "\u{2424}"),
            )
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
