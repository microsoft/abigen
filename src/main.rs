use abigen::ApiHeader;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
struct Args {
    pub header: PathBuf,
}

fn main() {
    let args = Args::parse();
    let api = ApiHeader::from_file(&args.header).expect("Failed to load header");
}
