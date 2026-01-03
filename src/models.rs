#[derive(Debug)]
pub struct ClassInfo {
    pub name: String,
    pub methods: Vec<String>,
    pub properties: Vec<String>,
    pub parents: Vec<String>,
}

#[derive(Debug)]
pub struct FileReport {
    pub path: String,
    pub classes: Vec<ClassInfo>,
}
