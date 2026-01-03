# 🌍 Marco Polo
**A CLI Tool to Cartograph Codebases**

Marco Polo (`marco`) is a high-performance CLI tool written in **Rust** that scans your codebase and generates **Mermaid.js Class Diagrams**. It helps developers visualize structure and inheritance in large projects quickly.

## 🚀 Features
- **Fast Scanning**: Uses the `ignore` crate to traverse directories while respecting `.gitignore`.
- **Accurate Parsing**: Leverages `tree-sitter` for robust AST-based code analysis.
- **Visual Output**: Generates `.mmd` files ready for Mermaid.js rendering.
- **Multi-language Support**: 
  - [x] Python
  - [ ] Java (Coming soon)
  - [ ] C++ (Coming soon)

## 🛠️ Getting Started

### Prerequisites
- [Rust & Cargo](https://rustup.rs/) installed.

### Installation
```bash
git clone https://github.com/wseabra/marco_polo.git
cd marco_polo
cargo build --release
```

### Usage
Run the tool against any directory:
```bash
cargo run -- .
```

## 🤝 Contributing
We love contributions! Whether it's adding support for a new language, fixing a bug, or improving documentation, here's how you can help:

1. **Fork** the repository.
2. **Create a branch** for your feature: `git checkout -b feat/my-new-feature`.
3. **Commit** your changes using [Conventional Commits](https://www.conventionalcommits.org/): `feat: add Java support`.
4. **Push** to your branch and **open a Pull Request**.

### Development
Run tests to ensure everything is working:
```bash
cargo test
```

## 📜 License
This project is licensed under the [MIT License](LICENSE).
