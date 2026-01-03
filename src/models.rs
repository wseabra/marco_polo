#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationshipType {
    Inheritance, // <|--
    Composition, // *--
    Aggregation, // o--
    Dependency,  // ..>
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Relationship {
    pub target: String,
    pub rel_type: RelationshipType,
    pub label: Option<String>,
}

#[derive(Debug)]
pub struct ClassInfo {
    pub name: String,
    pub methods: Vec<String>,
    pub properties: Vec<String>,
    pub relationships: Vec<Relationship>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct FileReport {
    pub path: std::path::PathBuf,
    pub classes: Vec<ClassInfo>,
}
