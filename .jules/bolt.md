# Bolt's Journal

## 2024-05-22 - [Rust Optimization Patterns]
**Learning:** In Rust, avoid unnecessary cloning of strings and vectors, especially in hot loops. Use references and iterators where possible.
**Action:** Look for `.clone()` calls in `src/` and see if they can be replaced with references or `Cow`.
