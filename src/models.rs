use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationshipType {
    Inheritance, // <|--
    Composition, // *--
    Aggregation, // o--
    Dependency,  // ..>
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Public,    // +
    Protected, // #
    Private,   // -
    Internal,  // ~
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Relationship {
    pub target: String,
    pub rel_type: RelationshipType,
    pub label: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodInfo {
    pub name: String,
    pub visibility: Visibility,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropertyInfo {
    pub name: String,
    pub visibility: Visibility,
}

#[derive(Debug)]
pub struct ClassInfo {
    pub name: String,
    pub methods: Vec<MethodInfo>,
    pub properties: Vec<PropertyInfo>,
    pub relationships: Vec<Relationship>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct FileReport {
    pub path: PathBuf,
    pub classes: Vec<ClassInfo>,
}