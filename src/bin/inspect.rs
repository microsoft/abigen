use abigen::{self, Context, NodeDisplayConfig};
use clap::Parser;
use std::{collections::HashMap, path::PathBuf};
use tree_sitter::{Query, QueryCursor, StreamingIterator};

/// Print the abstract syntax tree (AST) of a C++ header file
#[derive(Parser)]
struct Args {
    /// Path to a C++ header file
    header: PathBuf,

    /// Print AST nodes that match this S-expression instead of the file root node
    #[arg(short, long)]
    query: Option<String>,

    /// The number of descendant AST node layers to print
    #[arg(short = 'd', long, default_value_t = 5)]
    depth: usize,

    /// The amount of AST node code to print
    #[arg(short, long, default_value_t = 50)]
    code: usize,

    /// Show anonymous nodes
    #[arg(long, default_value_t = false)]
    anon: bool,
}

fn main() {
    let args = Args::parse();
    let ctx = Context::from_file(&args.header).expect("Failed to load file");

    NodeDisplayConfig {
        max_code: args.code,
        max_depth: args.depth,
        show_anon: args.anon,
    }
    .set()
    .expect("Failed to set formatting config");

    if let Some(ref qtext) = args.query {
        let q =
            Query::new(&tree_sitter_cpp::LANGUAGE.into(), qtext).expect("Failed to compile query");
        let capture_names = q.capture_names();
        let captures: HashMap<_, _> = capture_names
            .iter()
            .map(|name| (*name, q.capture_index_for_name(*name).unwrap()))
            .collect();

        let mut qc = QueryCursor::new();
        let node = ctx.root();
        let mut qm = qc.matches(&q, node.node, ctx.as_str().as_bytes());
        while let Some(m) = qm.next() {
            for (name, i) in captures.iter() {
                if let Some(n) = m.nodes_for_capture_index(*i).next() {
                    let node = ctx.wrap(n);
                    println!("{name}: {node}");
                }
            }
        }
    } else {
        println!("{}", ctx.root());
    }
}
