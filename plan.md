# 🌍 Project: Marco Polo
**A CLI Tool to Cartograph Codebases**

## 📋 Overview
**Marco Polo** (`marco`) is a high-performance Command Line Interface (CLI) tool written in **Rust**. It scans a local codebase, parses source files, and generates a **Mermaid.js Class Diagram** (`.mmd`).

**Goal:** Help developers visualize large codebases quickly.
**Current Target:** Python (v1), followed by Java and C++.

## 🛠️ Tech Stack & Architecture
* **Language:** Rust 🦀
* **CLI Framework:** `clap`
* **File System:** `ignore` (for recursive walking respecting `.gitignore`)
* **Parsing:** `tree-sitter` (Abstract Syntax Tree parsing)
* **Error Handling:** `anyhow`

---

## 🤖 AI Agent Prompts (Original Sequence)

Use these prompts sequentially to guide your AI coding assistant.

### Phase 0: Project Setup
*Action: Update `Cargo.toml` manually first.*

```toml
[dependencies]
clap = { version = "4.4", features = ["derive"] }
anyhow = "1.0"
ignore = "0.4"
tree-sitter = "0.20"
tree-sitter-python = "0.20"

```
### Prompt 1: The Skeleton (Data Structures)

 **Act as a Senior Rust Developer.** I am building a CLI tool called 'Marco Polo' that scans codebases and generates Mermaid class diagrams.
 First, please write the Rust `structs` to represent the data we need.
 I need a `ClassInfo` struct that holds:
 1. The class name (String).
 2. A list of public methods (Vec<String>).
 3. A list of public properties/fields (Vec<String>).
 4. A list of parent classes (Vec<String>) for inheritance.
 
 
 Also create a `FileReport` struct that holds the file path and a list of `ClassInfo` found in that file. Derive `Debug` for all of them so we can print them easily.

### Prompt 2: The Navigator (File Walking)

 Now, let's implement the file discovery module. We have decided to use the `ignore` crate instead of `walkdir` to respect gitignore files automatically.
 Please write a function called `find_source_files`.
 **Requirements:**
 1. Arguments: It takes a `Path` (root directory) and a list of extensions (e.g., `["py"]`) to filter by.
 2. Implementation: Use `ignore::WalkBuilder::new(path)` to create the walker. This automatically handles `.gitignore`.
 3. Filtering: Iterate through the entries.
 * Check if it is a file (not a directory).
 * Check if the file extension matches one of our allowed extensions (e.g., `.py`).
 
 
 4. Return: `anyhow::Result<Vec<PathBuf>>`.
 
 
 Please write this function and a small `main` function to test it by printing all `.py` files in the current directory, ignoring those in hidden or ignored folders.

### Prompt 3: The Cartographer (Tree-sitter Setup)

 Great. Now we need to parse these files. We are using `tree-sitter` and `tree-sitter-python`.
 Write a function `parse_python_file(content: &str) -> anyhow::Result<Vec<ClassInfo>>`.
 For now, do not worry about the exact Tree-sitter Query syntax (we will tune that next). Just set up the parser:
 1. Initialize a `Parser`.
 2. Set the language to `tree_sitter_python::language()`.
 3. Parse the `content` string to get a `Tree`.
 4. Return an empty vector for now.
 
 
 Explain to me how the `root_node()` works in Tree-sitter.

### Prompt 4: The Query (The Logic)

 Now implement the logic inside `parse_python_file`. Use a Tree-sitter `Query` to extract class definitions.
 The Query should look for:
 * Class declarations (`class_definition`).
 * Function definitions inside classes (`function_definition`).
 
 
 **Important Logic:**
 * Only include methods that do **not** start with `_` (underscore), as we only want public methods.
 * Extract the name of the class and the names of the methods.
 * Map the results into the `ClassInfo` structs we defined earlier.
 
 

### Prompt 5: The Illustrator (Mermaid Output)

 Finally, create a module to generate the Mermaid output.
 Write a function `generate_mermaid(classes: Vec<ClassInfo>) -> String`.
 Output format requirement:
 * Start with `classDiagram`.
 * For each class, write: `class Name { \n +method() \n }`.
 * If a class has parents, add the line: `Parent <|-- Child`.
 
 
 Return the final string.
