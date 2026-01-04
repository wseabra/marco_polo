use clap::Parser;
use std::path::PathBuf;
use anyhow::Result;
use std::fs;
use crate::models::Visibility;

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

    /// Visibility levels to include (comma-separated: public,protected,private)
    #[arg(short, long, value_delimiter = ',', default_value = "public")]
    visibility: Vec<String>,
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
        let ext = file_path.extension().and_then(|s| s.to_str()).unwrap_or("");
        
        if let Some(parser) = parsers::get_parser(ext) {
            eprintln!("Parsing: {:?}", file_path);
            let content = fs::read_to_string(&file_path)?;
            let classes = parser.parse(&content)?;
            all_classes.extend(classes);
        } else {
            eprintln!("Skipping {:?}: No parser found for extension '{}'", file_path, ext);
        }
    }

    eprintln!("Extracted {} classes.", all_classes.len());

    // 3. Map Visibility Strings to Enum
    let enabled_visibilities: Vec<Visibility> = args.visibility.iter()
        .map(|v| match v.to_lowercase().as_str() {
            "protected" => Visibility::Protected,
            "private" => Visibility::Private,
            _ => Visibility::Public,
        })
        .collect();

    // 4. Generate Diagram
    let diagram = mermaid::generate_mermaid(&all_classes, &enabled_visibilities);

    // 5. Write Output
    fs::write(&args.output, diagram)?;
    eprintln!("Successfully wrote Mermaid diagram to {:?}", args.output);

    Ok(())
}
