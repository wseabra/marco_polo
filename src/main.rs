use clap::Parser;
use std::path::PathBuf;
use anyhow::Result;
use std::fs;

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
    #[arg(short, long, value_delimiter = ',', default_value = "py")]
    extensions: Vec<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    eprintln!("Scanning path: {:?}", args.path);

    // 1. Find Files
    let extensions: Vec<&str> = args.extensions.iter().map(|s| s.as_str()).collect();
    let files = scanner::find_source_files(&args.path, &extensions)?;
    eprintln!("Found {} files with extensions {:?}.", files.len(), extensions);

    let mut all_classes = Vec::new();

    // 2. Parse Each File
    for file_path in files {
        eprintln!("Parsing: {:?}", file_path);
        let content = fs::read_to_string(&file_path)?;
        
        // TODO: Select parser based on extension (currently hardcoded to Python)
        let classes = parsers::python::parse_python_file(&content)?;
        all_classes.extend(classes);
    }

    eprintln!("Extracted {} classes.", all_classes.len());

    // 3. Generate Diagram
    let diagram = mermaid::generate_mermaid(&all_classes);

    // 4. Write Output
    fs::write(&args.output, diagram)?;
    eprintln!("Successfully wrote Mermaid diagram to {:?}", args.output);

    Ok(())
}
