use clap::Parser;
use std::path::PathBuf;
use anyhow::Result;
use std::fs;
use crate::models::Visibility;
use rayon::prelude::*;

mod models;
mod scanner;
mod parsers;
mod mermaid;

#[derive(Parser, Debug)]
#[command(author, version, about = "A CLI tool to cartograph codebases", long_about = None)]
struct Args {
    /// Path to the codebase to scan
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Output file path for the Mermaid diagram
    #[arg(short, long, default_value = "output.mmd")]
    output: PathBuf,

    /// File extensions to include (comma-separated)
    #[arg(short, long, value_delimiter = ',', default_value = "py,java,cpp,rb")]
    extensions: Vec<String>,

    /// Visibility levels to include (comma-separated: public,protected,private,internal)
    #[arg(short, long, value_delimiter = ',', default_values_t = vec![Visibility::Public])]
    visibility: Vec<Visibility>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    eprintln!("Scanning path: {:?}", args.path);

    // 1. Find Files
    let extensions: Vec<&str> = args.extensions.iter().map(|s| s.as_str()).collect();
    let files = scanner::find_source_files(&args.path, &extensions)?;
    eprintln!("Found {} files with extensions {:?}.", files.len(), extensions);

    // 2. Parse Each File (Parallel)
    // We use map to preserve Results and collect to stop on first error (fail-fast)
    let results: Result<Vec<Vec<models::ClassInfo>>> = files.par_iter()
        .map(|file_path| {
            let ext = file_path.extension().and_then(|s| s.to_str()).unwrap_or("");

            if let Some(parser) = parsers::get_parser(ext) {
                // Note: eprintln! inside parallel loop might be interleaved, but acceptable for progress
                // eprintln!("Parsing: {:?}", file_path);
                let content = fs::read_to_string(file_path)?;
                parser.parse(&content)
            } else {
                // Skip files without parsers
                Ok(Vec::new())
            }
        })
        .collect();

    let all_classes = results?.into_iter().flatten().collect::<Vec<_>>();

    eprintln!("Extracted {} classes.", all_classes.len());

    // 3. Generate Diagram
    let diagram = mermaid::generate_mermaid(&all_classes, &args.visibility);

    // 5. Write Output
    fs::write(&args.output, diagram)?;
    eprintln!("Successfully wrote Mermaid diagram to {:?}", args.output);

    Ok(())
}