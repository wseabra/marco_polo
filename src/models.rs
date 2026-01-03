#[derive(Debug)]
pub struct ClassInfo {
    pub name: String,
    pub methods: Vec<String>,
    pub properties: Vec<String>,
    pub parents: Vec<String>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct FileReport {
    pub path: std::path::PathBuf,
    pub classes: Vec<ClassInfo>,
}
