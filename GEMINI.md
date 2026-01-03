# Marco Polo (Project Context)

**Marco Polo** is a CLI tool written in Rust designed to "cartograph" codebases. It scans source code files, parses them, and generates visual Class Diagrams (specifically compatible with **Mermaid.js**).

## 📂 Project Structure

The project follows a standard Rust binary structure:

- **`src/`**
  - **`main.rs`**: Entry point. Handles CLI argument parsing via `clap` and orchestrates the application flow.
  - **`models.rs`**: Contains the core data structures (`ClassInfo`, `FileReport`) representing the parsed code metadata.
  - **`scanner.rs`**: Handles file system traversal and discovery, utilizing the `ignore` crate to respect `.gitignore` rules.
- **`tests/`**: Integration and unit test resources.
  - **`python/`**: Contains sample Python files used for verifying the scanner and parser.

## 🛠️ Tech Stack & Key Dependencies

- **Language**: Rust
- **CLI Framework**: [`clap`](https://crates.io/crates/clap) (v4.4)
- **Error Handling**: [`anyhow`](https://crates.io/crates/anyhow)
- **File System**: [`ignore`](https://crates.io/crates/ignore) (Efficient recursive directory iterator)
- **Parsing**:
  - [`tree-sitter`](https://crates.io/crates/tree-sitter) (Incremental parsing system)
  - [`tree-sitter-python`](https://crates.io/crates/tree-sitter-python) (Python grammar)

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
cargo run -- .
# OR
cargo run -- /path/to/codebase
```
*Current output matches `.py` files found in the target directory.*

### Test
Run the test suite (includes scanner verification):
```bash
cargo test
```

## 📝 Development Notes

- **File Discovery**: The scanner currently filters for specific extensions (e.g., `["py"]`) and respects `.gitignore`.
- **Parsing Status**: Dependencies are installed (`tree-sitter`), but the parsing logic implementation is the next immediate roadmap item.
- **Output**: Currently prints found files to stdout. Future goal is to generate `.mmd` (Mermaid) files.

## 🤝 Contribution Guidelines

### Commit Messages
We strictly follow **Conventional Commits** for all commit messages.
Format: `<type>(<scope>): <subject>`

### Branching and Merging
- Always work in feature branches.
- **NEVER** merge a Pull Request without explicit command/approval from the user.

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
