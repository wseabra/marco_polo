# Bolt's Journal
## 2026-01-06 - [Rust Parallel Iterator Error Handling]
**Learning:** When parallelizing loops with `rayon`, `collect()` can be used to handle `Result` types. Collecting into `Result<Vec<T>, E>` enables fail-fast behavior, stopping the parallel execution if any item returns an error.
**Action:** Always check if sequential loops rely on fail-fast behavior before blindly converting to `par_iter().for_each()`. Use `map()` + `collect::<Result<...>>` to preserve error propagation.
