# Marco (Project Context)

**Marco** is a CLI tool written in Rust designed to "cartograph" codebases. It scans source code files, parses them, and generates visual Class Diagrams (specifically compatible with **Mermaid.js**).

## 📂 Project Structure

The project follows a standard Rust binary structure:

- **`src/`**
  - **`main.rs`**: Entry point. Handles CLI argument parsing via `clap` and orchestrates the application flow.
  - **`models.rs`**: Contains the core data structures (`ClassInfo`, `Relationship`) representing the parsed code metadata.
  - **`scanner.rs`**: Handles file system traversal and discovery, utilizing the `ignore` crate to respect `.gitignore` rules.
  - **`mermaid.rs`**: Generates Mermaid.js class diagram strings from extracted metadata.
  - **`parsers/`**: Language-specific parsing logic.
    - **`mod.rs`**: Defines the `LanguageParser` trait and factory.
    - **`python.rs`**: Python implementation using tree-sitter.
    - **`java.rs`**: Java implementation using tree-sitter.
    - **`ruby.rs`**: Ruby implementation using tree-sitter.
- **`tests/`**: Integration and unit test resources.
  - **`python/`**: Sample Python files.
  - **`java/`**: Sample Java files.
  - **`ruby/`**: Sample Ruby files.

## 🛠️ Tech Stack & Key Dependencies

- **Language**: Rust
- **CLI Framework**: [`clap`](https://crates.io/crates/clap) (v4.5)
- **Error Handling**: [`anyhow`](https://crates.io/crates/anyhow)
- **File System**: [`ignore`](https://crates.io/crates/ignore) (Efficient recursive directory iterator)
- **Parsing**:
  - [`tree-sitter`](https://crates.io/crates/tree-sitter) (Incremental parsing system)
  - [`tree-sitter-python`](https://crates.io/crates/tree-sitter-python)
  - [`tree-sitter-java`](https://crates.io/crates/tree-sitter-java)
  - [`tree-sitter-cpp`](https://crates.io/crates/tree-sitter-cpp)
  - [`tree-sitter-ruby`](https://crates.io/crates/tree-sitter-ruby)

## 🚀 Building and Running

### Prerequisites
- Rust (Cargo) installed.

### Build
```bash
cargo build
```

### Run
To scan the current directory (or a specific path):
```bash
marco .
# OR
cargo run -- /path/to/codebase
```
*Default output matches `.py`, `.java`, `.cpp`, and `.rb` files found in the target directory.*

### Test
Run the test suite:
```bash
cargo test
```

## 📝 Development Notes

- **File Discovery**: The scanner filters for specific extensions (default: `["py", "java", "cpp", "rb"]`) and respects `.gitignore`.
- **Parsing Status**: Parsing logic is implemented for Python, Java, C++, and Ruby using `tree-sitter`.
    - **Visibility**: Supports `public`, `protected`, `private`, and `internal` (language-specific heuristics).
    - **Namespaces**: Detects nested classes and namespaces/packages.
    - **Relationships**: Detects inheritance, composition, aggregation, and dependency with automated labels.
- **Output**: Generates `.mmd` (Mermaid) files with configurable visibility levels.

## 🤝 Contribution Guidelines

### Commit Messages
We strictly follow **Conventional Commits** for all commit messages.
Format: `<type>(<scope>): <subject>`

### Branching and Merging
- Always work in feature branches.
- **NEVER** merge a Pull Request without explicit command/approval from the user.
- **NEVER** push directly to the `main` branch. All changes must go through a Pull Request.
- **ALWAYS** delete local and remote branches after merging a Pull Request.

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Formatting, missing semi-colons, etc; no code change
- `refactor`: Refactoring production code
- `test`: Adding missing tests, refactoring tests
- `chore`: Updating build tasks, package manager configs, etc.

**Example:**
`feat(scanner): add support for excluding hidden files`