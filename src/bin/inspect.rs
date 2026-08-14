use std::path::PathBuf;
use clap::Parser;
use tree_sitter::{Query, QueryCursor, StreamingIterator};
use abigen::{self, Context, InContext};

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
    max_depth: u32,

    /// The amount of AST node code to print
    #[arg(short, long, default_value_t = 50)]
    max_code: usize,
}

fn main() {
    let args = Args::parse();
    let ctx = Context::try_from_file(&args.header)
        .expect("Failed to load file");

    if let Some(ref qtext) = args.query {
        let q = Query::new(&abigen::LANG, qtext).expect("Failed to compile query"); 
        let mut qc = QueryCursor::new();
        let node = ctx.root();
        let mut qm = node.matches(&q, &mut qc);
        while let Some(m) = qm.item.next() {
            let node = InContext::new(m.nodes_for_capture_index(0).next().unwrap(), &ctx);
            abigen::print_node(node, args.max_depth, args.max_code);
            println!();
        }
    } else {
        abigen::print_node(ctx.root(), args.max_depth, args.max_code);
    }
}
