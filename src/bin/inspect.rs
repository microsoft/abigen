use abigen::{self, Context};
use clap::Parser;
use std::path::PathBuf;
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
    depth: u32,

    /// The amount of AST node code to print
    #[arg(short, long, default_value_t = 50)]
    code: usize,

    /// Show anonymous nodes
    #[arg(long)]
    anon: bool,
}

fn main() {
    let args = Args::parse();
    let ctx = Context::from_file(&args.header).expect("Failed to load file");

    if let Some(ref qtext) = args.query {
        let q =
            Query::new(&tree_sitter_cpp::LANGUAGE.into(), qtext).expect("Failed to compile query");
        let mut qc = QueryCursor::new();
        let node = ctx.root();
        let mut qm = qc.matches(&q, node.node, ctx.text());
        while let Some(m) = qm.next() {
            let node = ctx.wrap(m.nodes_for_capture_index(0).next().unwrap());
            abigen::print_node(node, args.depth, args.code, args.anon);
            println!();
        }
    } else {
        abigen::print_node(ctx.root(), args.depth, args.code, args.anon);
    }
}
