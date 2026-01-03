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

        // Expect at least animals.py
        let found_animals = files.iter().any(|p| p.file_name().unwrap() == "animals.py");
        assert!(found_animals, "Should find animals.py");

        // Expect NOT to find ignore_me.txt
        let found_txt = files.iter().any(|p| p.file_name().unwrap() == "ignore_me.txt");
        assert!(!found_txt, "Should not find ignore_me.txt");

        Ok(())
    }
}