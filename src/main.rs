use std::path::PathBuf;
use clap::Parser;
use abigen::{ApiSurface};

#[derive(Parser)]
struct Args {
    pub header: PathBuf,
}

fn main() {
    let args = Args::parse();
    let mut api = ApiSurface::new();
    api.add_header(&args.header).expect("Failed to load header");
}
