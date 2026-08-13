use std::{fmt::Display, fs, path::PathBuf};
use clap::Parser;
use tree_sitter::{Node, TreeCursor};

#[derive(Parser)]
struct Args {
    /// Path to a C++ header file
    header: PathBuf,
}

fn main() {
    let args = Args::parse();
    let buf = fs::read(args.header).expect("Failed to load header");

    let mut p = tree_sitter::Parser::new();
    p.set_language(&tree_sitter_cpp::LANGUAGE.into()).expect("failed to set lang");

    let tree = p.parse(&buf, None).expect("no tree");
    let root = tree.root_node();

    print_node(root, &buf,  5);
}

fn print_node(node: Node, buf: impl AsRef<[u8]>, max_depth: u32) {
    print_tree(node.walk(), buf.as_ref(), max_depth);
}

fn print_tree<'a>(mut cursor: TreeCursor<'a>, buf: &[u8], max_depth: u32) -> TreeCursor<'a> {
    if !cursor.node().is_named() || cursor.depth() > max_depth {
        return cursor;
    }

    for _ in 0..cursor.depth() {
        print!("  ");
    }
    println!("{}",
        NodeFormat::<50> {
            node: cursor.node(),
            buf,
            field: cursor.field_name(),
        },
    );

    if cursor.goto_first_child() {
        cursor = print_tree(cursor, buf, max_depth);
        while cursor.goto_next_sibling() {
            cursor = print_tree(cursor, buf, max_depth);
        }
        cursor.goto_parent();
    }

    cursor
}

struct NodeFormat<'a, const CMAX: usize> {
    pub node: Node<'a>, 
    pub buf: &'a [u8],
    pub field: Option<&'static str>,
}

impl<const CMAX: usize> Display for NodeFormat<'_, CMAX> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let body = if self.node.byte_range().len() > CMAX {
            format!("\"{}\" +{} bytes",
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
