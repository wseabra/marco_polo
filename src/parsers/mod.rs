use crate::models::ClassInfo;
use anyhow::Result;

pub mod python;
pub mod java;
pub mod cpp;
pub mod ruby;

pub trait LanguageParser {
    /// The file extensions this parser handles (e.g., ["py", "py3"])
    #[allow(dead_code)]
    fn extensions(&self) -> &[&str];

    /// The core parsing logic
    fn parse(&self, content: &str) -> Result<Vec<ClassInfo>>;
}

pub fn get_parser(extension: &str) -> Option<Box<dyn LanguageParser>> {
    match extension {
        "py" => Some(Box::new(python::PythonParser)),
        "java" => Some(Box::new(java::JavaParser)),
        "cpp" | "cc" | "cxx" | "h" | "hpp" => Some(Box::new(cpp::CppParser)),
        "rb" => Some(Box::new(ruby::RubyParser)),
        _ => None,
    }
}
