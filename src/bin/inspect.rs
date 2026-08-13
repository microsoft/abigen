use std::{fs, path::PathBuf};
use clap::Parser;
use abigen::{Context, format::print_node};

#[derive(Parser)]
struct Args {
    /// Path to a C++ header file
    header: PathBuf,

    /// Only print AST nodes that match this query
    #[arg(short, long)]
    query: Option<String>,

    /// How deep 
    #[arg(short = 'd', long, default_value_t = 5)]
    max_depth: u32,

    #[arg(short, long, default_value_t = 50)]
    context: usize,
}

fn main() {
    let args = Args::parse();
    let ctx = Context::try_from_file(&args.header)
        .expect("Failed to load file");

    if let Some(ref qtext) = args.query {
        let q = Query::new(&abigen::LANG, qtext).expect("Failed to compile query"); 
        let mut qc = QueryCursor::new();
        let mut qm = qc.matches(&q, root, &buf as &[u8]);
        while let Some(m) = qm.next() {
            print_node(m.nodes_for_capture_index(0).next().unwrap(), &buf, args.max_depth);
            println!();
        }
    } else {
        print_node(root, &buf,  args.max_depth);
    }
}
