use std::{fmt::Display, fs, path::PathBuf, sync::LazyLock};
use tree_sitter::{Language, Node, Query, QueryCursor, StreamingIterator, TreeCursor};

const INDENT: &str = "  ";

pub fn print_node(node: Node, buf: impl AsRef<[u8]>, max_depth: u32) {
    print_tree(node.walk(), buf.as_ref(), max_depth);
}

pub fn print_tree<'a>(mut cursor: TreeCursor<'a>, buf: &[u8], max_depth: u32) -> TreeCursor<'a> {
    if !cursor.node().is_named() {
        return cursor;
    }

    for _ in 0..cursor.depth() {
        print!("{INDENT}");
    }

    println!("{}",
        NodeFormat::<50> {
            node: cursor.node(),
            buf,
            field: cursor.field_name(),
        },
    );

    if cursor.depth() == max_depth - 1 && cursor.node().named_child_count() > 0 {
        for _ in 0 .. max_depth {
            print!("{INDENT}");
        }
        println!("+{} children", cursor.node().named_child_count());
    } else if cursor.goto_first_child() {
        cursor = print_tree(cursor, buf, max_depth);
        while cursor.goto_next_sibling() {
            cursor = print_tree(cursor, buf, max_depth);
        }
        cursor.goto_parent();
    }

    cursor
}

pub struct NodeFormat<'a, const CMAX: usize> {
    pub node: Node<'a>, 
    pub buf: &'a [u8],
    pub field: Option<&'static str>,
}

impl<const CMAX: usize> Display for NodeFormat<'_, CMAX> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let body = if self.node.byte_range().len() > CMAX {
            format!("\"{}...\" +{} bytes",
                str::from_utf8(&self.buf[self.node.start_byte() .. self.node.start_byte() + CMAX])
                    .map_err(|_| std::fmt::Error)?
                    .replace('\n', "\u{2424}"),
                self.node.byte_range().len() - CMAX,
            )
        } else {
            format!("\"{}\"",
                self.node.utf8_text(self.buf)
                    .map_err(|_| std::fmt::Error)?
                    .replace('\n', "\u{2424}"),
            )
        };

        match self.field {
            Some(field) => {
                write!(f, "({field}) {typ} {body}", typ = self.node.kind())
            },
            None => {
                write!(f, "{typ} {body}", typ = self.node.kind())
            },
        }
    }
}
