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

    #[test]
    fn test_find_python_files() -> Result<()> {
        let root = Path::new("tests/python");
        let files = find_source_files(root, &["py"])?;

        // Should find at least `tests/python/animals.py`.
        assert!(files.len() >= 1, "Should find at least one .py file");
        assert!(
            files.iter().any(|p| p.ends_with("tests/python/animals.py")),
            "The found files should include animals.py"
        );

        Ok(())
    }

    #[test]
    fn test_find_cpp_files() -> Result<()> {
        let root = Path::new("tests/cpp");
        let files = find_source_files(root, &["cpp"])?;

        // Should find at least `tests/cpp/Animals.cpp`.
        assert!(files.len() >= 1, "Should find at least one .cpp file");
        assert!(
            files.iter().any(|p| p.ends_with("tests/cpp/Animals.cpp")),
            "The found files should include Animals.cpp"
        );

        Ok(())
    }

    #[test]
    fn test_find_ruby_files() -> Result<()> {
        let root = Path::new("tests/ruby");
        let files = find_source_files(root, &["rb"])?;

        assert!(files.len() >= 1, "Should find at least one .rb file");
        assert!(
            files.iter().any(|p| p.ends_with("tests/ruby/animals.rb")),
            "The found files should include animals.rb"
        );

        Ok(())
    }
}
        