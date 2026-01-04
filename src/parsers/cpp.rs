use std::sync::OnceLock;
use tree_sitter::{Parser, Query, QueryCursor, Node};
use crate::models::{ClassInfo, Relationship, RelationshipType};
use anyhow::{Result, Context};
use super::LanguageParser;

const CPP_CLASS_QUERY_STR: &str = "
    (class_specifier) @class
";

pub struct CppParser;

impl LanguageParser for CppParser {
    fn extensions(&self) -> &[&str] {
        &["cpp", "cc", "cxx", "h", "hpp"]
    }

    fn parse(&self, content: &str) -> Result<Vec<ClassInfo>> {
        let mut parser = Parser::new();
        let language = tree_sitter_cpp::language();
        parser.set_language(language)
            .context("Error loading C++ grammar")?;

        let tree = parser.parse(content, None)
            .context("Failed to parse C++ content")?;

        let root_node = tree.root_node();
        let mut classes = Vec::new();

        static CLASS_QUERY: OnceLock<Query> = OnceLock::new();
        let query = CLASS_QUERY.get_or_init(|| {
            Query::new(tree_sitter_cpp::language(), CPP_CLASS_QUERY_STR)
                .expect("Static C++ class query is invalid")
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

            // 1. Inheritance (base class clause)
            // Note: base_class_clause is not a named field in all grammar versions, so we search by kind
            if let Some(bases_node) = find_node_by_kind(class_node, "base_class_clause") {
                extract_inheritance(bases_node, content, &mut relationships);
            }

            // 2. Body: Fields and Methods
            if let Some(body_node) = class_node.child_by_field_name("body") {
                let mut cursor = body_node.walk();
                for child in body_node.children(&mut cursor) {
                    match child.kind() {
                        "field_declaration" => {
                            // First check if this is actually a method (e.g., pure virtual or function pointer)
                            if let Some(declarator) = child.child_by_field_name("declarator") {
                                if find_function_declarator(declarator).is_some() {
                                    // Treat as method
                                    if let Some(name_node) = find_node_by_kind(declarator, "field_identifier")
                                        .or_else(|| find_node_by_kind(declarator, "identifier")) {
                                        let method_name = get_node_text(name_node, content);
                                        methods.push(method_name.clone());
                                        
                                        // Extract parameter types for dependency relationships
                                        if let Some(func_decl) = find_function_declarator(declarator) {
                                            if let Some(params) = find_node_by_kind(func_decl, "parameter_list") {
                                                extract_parameter_types(params, content, &mut relationships);
                                            }
                                        }

                                        // Extract return type for dependency
                                        extract_return_type(child, content, &mut relationships);

                                        continue;
                                    }
                                }

                                // Otherwise treat as property
                                // Find the identifier deeply within the declarator (handles pointers, refs, arrays)
                                if let Some(field_id) = find_node_by_kind(declarator, "field_identifier")
                                    .or_else(|| find_node_by_kind(declarator, "identifier")) {
                                    
                                    let field_name = get_node_text(field_id, content);
                                    properties.push(field_name.clone());

                                    // Get the type from the 'type' field of the field_declaration
                                    if let Some(type_node) = child.child_by_field_name("type") {
                                        let mut type_nodes = Vec::new();
                                        extract_type(type_node, content, &mut type_nodes);
                                        
                                        // Check for composition (initialized with constructor)
                                        let is_composition = has_initializer(declarator);
                                        
                                        // Check for aggregation (pointer or reference)
                                        // We need to check if the declarator WRAPPERS are pointers/refs
                                        let is_pointer_or_ref = is_pointer_or_reference_wrapper(declarator);
                                        
                                        let rel_type = if is_composition {
                                            RelationshipType::Composition
                                        } else if is_pointer_or_ref {
                                            RelationshipType::Aggregation
                                        } else {
                                            // Regular member variable could be composition or aggregation
                                            RelationshipType::Composition
                                        };

                                        for type_name in type_nodes {
                                            relationships.push(Relationship {
                                                target: type_name,
                                                rel_type: rel_type.clone(),
                                                label: Some(field_name.clone()),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                        "function_definition" | "declaration" => {
                            // Check if it's a method declaration
                            if let Some(declarator) = find_function_declarator(child) {
                                if let Some(name_node) = find_node_by_kind(declarator, "field_identifier")
                                    .or_else(|| find_node_by_kind(declarator, "identifier")) {
                                    let method_name = get_node_text(name_node, content);
                                    
                                    // Skip constructors and destructors from methods list
                                    if !method_name.starts_with('~') && method_name != name {
                                        methods.push(method_name.clone());
                                    }

                                    // Extract parameter types for dependency relationships
                                    if let Some(params) = find_node_by_kind(declarator, "parameter_list") {
                                        extract_parameter_types(params, content, &mut relationships);
                                    }

                                    // Extract return type for dependency
                                    extract_return_type(child, content, &mut relationships);
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
        if child.kind() == "type_identifier" || child.kind() == "qualified_identifier" {
            let parent = get_node_text(child, content);
            relationships.push(Relationship {
                target: parent,
                rel_type: RelationshipType::Inheritance,
                label: None,
            });
        } else {
            // Recursively search
            extract_inheritance(child, content, relationships);
        }
    }
}

fn extract_type(node: Node, content: &str, types: &mut Vec<String>) {
    match node.kind() {
        "type_identifier" | "qualified_identifier" => {
             let type_name = get_node_text(node, content);
             if !is_builtin_type(&type_name) {
                 types.push(type_name);
             }
        }
        "primitive_type" => {} // Ignore
        "template_type" => {
             // Handle std::vector<Type> -> extract Type
             // Template type has name (type_identifier) and arguments (template_argument_list)
             if let Some(args) = node.child_by_field_name("arguments") {
                 let mut cursor = args.walk();
                 for child in args.children(&mut cursor) {
                     extract_type(child, content, types);
                 }
             }
        }
        _ => {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                extract_type(child, content, types);
            }
        }
    }
}

fn extract_parameter_types(params_node: Node, content: &str, relationships: &mut Vec<Relationship>) {
    let mut cursor = params_node.walk();
    for child in params_node.children(&mut cursor) {
        if child.kind() == "parameter_declaration" {
            if let Some(type_node) = child.child_by_field_name("type") {
                 let mut types = Vec::new();
                 extract_type(type_node, content, &mut types);
                 for type_name in types {
                    relationships.push(Relationship {
                        target: type_name,
                        rel_type: RelationshipType::Dependency,
                        label: None,
                    });
                }
            }
        }
    }
}

fn extract_return_type(node: Node, content: &str, relationships: &mut Vec<Relationship>) {
    // Return type is the 'type' field of function_definition
    if let Some(type_node) = node.child_by_field_name("type") {
         let mut types = Vec::new();
         extract_type(type_node, content, &mut types);
         for type_name in types {
             if type_name != "void" {
                relationships.push(Relationship {
                    target: type_name,
                    rel_type: RelationshipType::Dependency,
                    label: None,
                });
             }
         }
    }
}

fn has_initializer(declarator: Node) -> bool {
    // Check if the declarator is an 'init_declarator', meaning it has an initializer.
    declarator.kind() == "init_declarator"
}

fn is_pointer_or_reference_wrapper(node: Node) -> bool {
    // The declarator might be a pointer_declarator wrapping an identifier
    // or a reference_declarator wrapping an identifier
    match node.kind() {
        "pointer_declarator" | "reference_declarator" => {
            return true;
        }
        _ => {
            // Check usage in init_declarator?
            // Usually init_declarator( declarator: pointer_declarator(...) )
            if let Some(child) = node.child_by_field_name("declarator") {
                return is_pointer_or_reference_wrapper(child);
            }
            false
        }
    }
}

fn find_node_by_kind<'a>(node: Node<'a>, kind: &str) -> Option<Node<'a>> {
    if node.kind() == kind {
        return Some(node);
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if let Some(found) = find_node_by_kind(child, kind) {
            return Some(found);
        }
    }
    None
}

fn find_function_declarator<'a>(node: Node<'a>) -> Option<Node<'a>> {
    if node.kind() == "function_declarator" {
        return Some(node);
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if let Some(found) = find_function_declarator(child) {
            return Some(found);
        }
    }
    None
}

fn is_builtin_type(type_name: &str) -> bool {
    matches!(
        type_name,
        "int" | "char" | "short" | "long" | "float" | "double" | "bool" | "void" |
        "unsigned" | "signed" | "size_t" | "uint8_t" | "uint16_t" | "uint32_t" | "uint64_t" |
        "int8_t" | "int16_t" | "int32_t" | "int64_t" |
        "std::string" | "std::vector" | "std::map" | "std::set" | "std::list" |
        "std::unique_ptr" | "std::shared_ptr" | "std::weak_ptr"
    )
}

fn get_node_text(node: Node, content: &str) -> String {
    node.utf8_text(content.as_bytes())
        .map(ToString::to_string)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_class() -> Result<()> {
        let content = "
class Animal {
public:
    std::string name;
    void speak() {}
};
";
        let classes = CppParser.parse(content)?;
        assert_eq!(classes.len(), 1);
        let animal = &classes[0];
        assert_eq!(animal.name, "Animal");
        assert!(animal.properties.contains(&"name".to_string()));
        assert!(animal.methods.contains(&"speak".to_string()));
        Ok(())
    }

    #[test]
    fn test_parse_inheritance() -> Result<()> {
        let content = "
class Animal {
public:
    void speak() {}
};

class Dog : public Animal {
public:
    void bark() {}
};
";
        let classes = CppParser.parse(content)?;
        assert_eq!(classes.len(), 2);
        let dog = classes.iter().find(|c| c.name == "Dog").unwrap();
        assert!(dog.relationships.iter().any(|r| r.target == "Animal" && r.rel_type == RelationshipType::Inheritance));
        Ok(())
    }

    #[test]
    fn test_parse_composition() -> Result<()> {
        let content = "
class Door {};

class House {
private:
    Door door;
};
";
        let classes = CppParser.parse(content)?;
        let house = classes.iter().find(|c| c.name == "House").unwrap();
        assert!(house.relationships.iter().any(|r| r.target == "Door" && r.rel_type == RelationshipType::Composition));
        Ok(())
    }

    #[test]
    fn test_parse_aggregation() -> Result<()> {
        let content = "
class Engine {};

class Car {
private:
    Engine* engine;
};
";
        let classes = CppParser.parse(content)?;
        let car = classes.iter().find(|c| c.name == "Car").unwrap();
        assert!(car.relationships.iter().any(|r| r.target == "Engine" && r.rel_type == RelationshipType::Aggregation));
        Ok(())
    }

    #[test]
    fn test_parse_dependency() -> Result<()> {
        let content = "
class Post {};

class Admin {
public:
    void deletePost(Post* post) {}
};
";
        let classes = CppParser.parse(content)?;
        let admin = classes.iter().find(|c| c.name == "Admin").unwrap();
        assert!(admin.relationships.iter().any(|r| r.target == "Post" && r.rel_type == RelationshipType::Dependency));
        Ok(())
    }

    #[test]
    fn test_parse_multiple_inheritance() -> Result<()> {
        let content = "
class Auth {};
class Loggable {};

class Admin : public Auth, public Loggable {
public:
    void log() {}
};
";
        let classes = CppParser.parse(content)?;
        let admin = classes.iter().find(|c| c.name == "Admin").unwrap();
        assert!(admin.relationships.iter().any(|r| r.target == "Auth" && r.rel_type == RelationshipType::Inheritance));
        assert!(admin.relationships.iter().any(|r| r.target == "Loggable" && r.rel_type == RelationshipType::Inheritance));
        Ok(())
    }

    #[test]
    fn test_parse_function_pointer_parameter_dependency() -> Result<()> {
        let content = "
class Dependency {};

class Handler {
    // Function pointer field
    void (*callback)(Dependency* d);
};
";
        let classes = CppParser.parse(content)?;
        let handler = classes.iter().find(|c| c.name == "Handler").unwrap();
        // Should find dependency on 'Dependency'
        assert!(handler.relationships.iter().any(|r| r.target == "Dependency" && r.rel_type == RelationshipType::Dependency));
        Ok(())
    }

    #[test]
    fn test_parse_function_pointer_return_type_dependency() -> Result<()> {
        let content = "
class ReturnType {};
class Handler2 {
    ReturnType* (*callback)();
};
";
        let classes = CppParser.parse(content)?;
        let handler2 = classes.iter().find(|c| c.name == "Handler2").unwrap();
        assert!(handler2.relationships.iter().any(|r| r.target == "ReturnType" && r.rel_type == RelationshipType::Dependency));
        Ok(())
    }
}
