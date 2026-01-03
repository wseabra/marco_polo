use std::path::{Path, PathBuf};
use ignore::WalkBuilder;
use anyhow::Result;

pub fn find_source_files(root: &Path, extensions: &[&str]) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for entry in WalkBuilder::new(root).build() {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                if extensions.contains(&ext) {
                    files.push(path.to_path_buf());
                }
            }
        }
    }

    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_find_python_files() -> Result<()> {
        // Setup is already done by the user (tests/python/animals.py exists)
        // But to be safe and self-contained, we can point to the project root or specific test dir
        
        let root = Path::new("tests/python");
        let files = find_source_files(root, &["py"])?;

        // Expect exactly one file: `tests/python/animals.py`.
        assert_eq!(files.len(), 1, "Should find exactly one .py file");
        assert!(
            files[0].ends_with("tests/python/animals.py"),
            "The found file should be animals.py"
        );

        Ok(())
    }
}