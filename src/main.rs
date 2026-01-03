use clap::Parser;
use std::path::PathBuf;
use anyhow::Result;

mod models;
mod scanner;
mod parsers;

#[derive(Parser, Debug)]
#[command(author, version, about = "A CLI tool to cartograph codebases", long_about = None)]
struct Args {
    /// Path to the codebase to scan
    #[arg(default_value = ".")]
    path: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();
    println!("Scanning path: {:?}", args.path);

    let files = scanner::find_source_files(&args.path, &["py"])?;
    
    println!("Found {} Python files:", files.len());
    for file in files {
        println!("  - {:?}", file);
    }

    Ok(())
}
