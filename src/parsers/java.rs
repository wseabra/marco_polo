use std::sync::OnceLock;
use tree_sitter::{Parser, Query, QueryCursor, Node};
use crate::models::{ClassInfo, Relationship, RelationshipType};
use anyhow::{Result, Context};
use super::LanguageParser;

const JAVA_CLASS_QUERY_STR: &str = "
    (class_declaration) @class
    (interface_declaration) @interface
";

pub struct JavaParser;

impl LanguageParser for JavaParser {
    fn extensions(&self) -> &[&str] {
        &["java"]
    }

    fn parse(&self, content: &str) -> Result<Vec<ClassInfo>> {
        let mut parser = Parser::new();
        let language = tree_sitter_java::language();
        parser.set_language(language)
            .context("Error loading Java grammar")?;

        let tree = parser.parse(content, None)
            .context("Failed to parse Java content")?;

        let root_node = tree.root_node();
        let mut classes = Vec::new();

        static CLASS_QUERY: OnceLock<Query> = OnceLock::new();
        let query = CLASS_QUERY.get_or_init(|| {
            Query::new(tree_sitter_java::language(), JAVA_CLASS_QUERY_STR)
                .expect("Static Java class query is invalid")
        });

        let mut query_cursor = QueryCursor::new();
        let matches = query_cursor.matches(query, root_node, content.as_bytes());

        for m in matches {
            let class_node = m.captures[0].node;
            
            let name = class_node.child_by_field_name("name")
                .map(|n| get_node_text(n, content))
                .unwrap_or_else(|| "Anonymous".to_string());

            let mut methods = Vec::new();
            let mut properties = Vec::new();
            let mut relationships = Vec::new();

            // 1. Inheritance (Classes and Interfaces)
            let mut inheritance_cursor = class_node.walk();
            for child in class_node.children(&mut inheritance_cursor) {
                match child.kind() {
                    "superclass" | "super_interfaces" | "extends_interfaces" => {
                        extract_inheritance(child, content, &mut relationships);
                    }
                    _ => {}
                }
            }

            // 2. Body: Fields and Methods
            if let Some(body_node) = class_node.child_by_field_name("body") {
                let mut cursor = body_node.walk();
                for child in body_node.children(&mut cursor) {
                    match child.kind() {
                        "field_declaration" => {
                            let type_node = child.child_by_field_name("type");
                            let mut cursor = child.walk();
                            for field_child in child.children(&mut cursor) {
                                if field_child.kind() == "variable_declarator" {
                                    if let Some(name_node) = field_child.child_by_field_name("name") {
                                        let field_name = get_node_text(name_node, content);
                                        properties.push(field_name.clone());

                                        if let Some(t_node) = type_node {
                                            let mut resolved = Vec::new();
                                            resolve_java_types(t_node, content, &mut resolved);
                                            
                                            // Check for Composition (new instantiation)
                                            let is_composition = field_child.child_by_field_name("value")
                                                .map(|v| v.kind() == "object_creation_expression")
                                                .unwrap_or(false);

                                            let rel_type = if is_composition {
                                                RelationshipType::Composition
                                            } else {
                                                RelationshipType::Aggregation
                                            };

                                            for t in resolved {
                                                relationships.push(Relationship {
                                                    target: t,
                                                    rel_type: rel_type.clone(),
                                                    label: Some(field_name.clone()),
                                                });
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        "method_declaration" | "constructor_declaration" => {
                            if let Some(name_node) = child.child_by_field_name("name") {
                                let method_name = get_node_text(name_node, content);
                                if child.kind() == "method_declaration" {
                                    methods.push(method_name);
                                }

                                // Parameters for Dependency/Aggregation
                                if let Some(params_node) = child.child_by_field_name("parameters") {
                                    let mut p_cursor = params_node.walk();
                                    for param in params_node.children(&mut p_cursor) {
                                        if param.kind() == "formal_parameter" {
                                            if let Some(type_node) = param.child_by_field_name("type") {
                                                let mut resolved = Vec::new();
                                                resolve_java_types(type_node, content, &mut resolved);
                                                for t in resolved {
                                                    let rel_type = if child.kind() == "constructor_declaration" {
                                                        RelationshipType::Aggregation
                                                    } else {
                                                        RelationshipType::Dependency
                                                    };
                                                    relationships.push(Relationship {
                                                        target: t,
                                                        rel_type,
                                                        label: None,
                                                    });
                                                }
                                            }
                                        }
                                    }
                                }

                                // Return type for Dependency
                                if let Some(ret_type_node) = child.child_by_field_name("type") {
                                    let mut resolved = Vec::new();
                                    resolve_java_types(ret_type_node, content, &mut resolved);
                                    for t in resolved {
                                        relationships.push(Relationship {
                                            target: t,
                                            rel_type: RelationshipType::Dependency,
                                            label: None,
                                        });
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }

            classes.push(ClassInfo {
                name,
                methods,
                properties,
                relationships,
            });
        }

        Ok(classes)
    }
}

fn extract_inheritance(node: Node, content: &str, relationships: &mut Vec<Relationship>) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "type_list" {
            let mut inner_cursor = child.walk();
            for type_node in child.children(&mut inner_cursor) {
                if type_node.kind().contains("type") {
                    let parent = get_node_text(type_node, content);
                    relationships.push(Relationship {
                        target: parent,
                        rel_type: RelationshipType::Inheritance,
                        label: None,
                    });
                }
            }
        } else if child.kind().contains("type") {
            let parent = get_node_text(child, content);
            relationships.push(Relationship {
                target: parent,
                rel_type: RelationshipType::Inheritance,
                label: None,
            });
        }
    }
}

fn resolve_java_types(node: Node, content: &str, types: &mut Vec<String>) {
    match node.kind() {
        "type_identifier" => {
            let name = get_node_text(node, content);
            match name.as_str() {
                "byte" | "short" | "int" | "long" | "float" | "double" | "char" | "boolean" | "void" |
                "String" | "Object" | "List" | "ArrayList" | "Map" | "HashMap" | "Set" | "HashSet" | "Optional" => {},
                _ => types.push(name),
            }
        }
        _ => {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                resolve_java_types(child, content, types);
            }
        }
    }
}

fn get_node_text(node: Node, content: &str) -> String {
    node.utf8_text(content.as_bytes())
        .ok()
        .unwrap_or("")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inspect_interface_tree() {
        let content = "public interface D extends A, B, C {}";
        let mut parser = Parser::new();
        parser.set_language(tree_sitter_java::language()).unwrap();
        let tree = parser.parse(content, None).unwrap();
        println!("DEBUG_JAVA_TREE: {}", tree.root_node().to_sexp());
    }

    #[test]
    fn test_parse_interface_inheritance() -> Result<()> {
        let content = "public interface D extends A, B, C {}";
        let classes = JavaParser.parse(content)?;
        let d = &classes[0];
        assert!(d.relationships.iter().any(|r| r.target == "A" && r.rel_type == RelationshipType::Inheritance));
        assert!(d.relationships.iter().any(|r| r.target == "B" && r.rel_type == RelationshipType::Inheritance));
        assert!(d.relationships.iter().any(|r| r.target == "C" && r.rel_type == RelationshipType::Inheritance));
        Ok(())
    }

    #[test]
    fn test_parse_java_simple() -> Result<()> {
        let content = "
public class User {
    private String name;
    public void speak() {}
}
";
        let classes = JavaParser.parse(content)?;
        assert_eq!(classes.len(), 1);
        let user = &classes[0];
        assert_eq!(user.name, "User");
        assert!(user.properties.contains(&"name".to_string()));
        assert!(user.methods.contains(&"speak".to_string()));
        Ok(())
    }

    #[test]
    fn test_parse_java_composition() -> Result<()> {
        let content = "
public class House {
    private Door door = new Door();
}
";
        let classes = JavaParser.parse(content)?;
        let house = &classes[0];
        assert!(house.relationships.iter().any(|r| r.target == "Door" && r.rel_type == RelationshipType::Composition));
        Ok(())
    }

    #[test]
    fn test_parse_java_relationships() -> Result<()> {
        let content = "
public class Admin extends User implements Auth, Loggable {
    private Logger logger;
    public Admin(Logger logger) {}
    public void delete(Post post) {}
}
";
        let classes = JavaParser.parse(content)?;
        let admin = &classes[0];
        
        let rels = &admin.relationships;
        
        // Inheritance
        assert!(rels.iter().any(|r| r.target == "User" && r.rel_type == RelationshipType::Inheritance));
        assert!(rels.iter().any(|r| r.target == "Auth" && r.rel_type == RelationshipType::Inheritance));
        assert!(rels.iter().any(|r| r.target == "Loggable" && r.rel_type == RelationshipType::Inheritance));
        
        // Aggregation (Field + Constructor)
        assert!(rels.iter().any(|r| r.target == "Logger" && r.rel_type == RelationshipType::Aggregation));
        
        // Dependency (Method param)
        assert!(rels.iter().any(|r| r.target == "Post" && r.rel_type == RelationshipType::Dependency));
        
        Ok(())
    }
}