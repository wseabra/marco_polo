use std::sync::OnceLock;
use tree_sitter::{Parser, Query, QueryCursor, Node};
use crate::models::{ClassInfo, Relationship, RelationshipType, Visibility, MethodInfo, PropertyInfo};
use anyhow::{Result, Context};
use std::collections::HashSet;
use super::LanguageParser;

const CLASS_QUERY_STR: &str = "(class_definition) @class";
const PROP_QUERY_STR: &str = "
    (assignment left: (attribute object: (identifier) @obj attribute: (identifier) @attr))
    (assignment left: (pattern_list (attribute object: (identifier) @obj attribute: (identifier) @attr)))
";

pub struct PythonParser;

impl LanguageParser for PythonParser {
    fn extensions(&self) -> &[&str] {
        &["py"]
    }

    fn parse(&self, content: &str) -> Result<Vec<ClassInfo>> {
        let mut parser = Parser::new();
        let language = tree_sitter_python::language();
        parser.set_language(language)
            .context("Error loading Python grammar")?;

        let tree = parser.parse(content, None)
            .context("Failed to parse Python content")?;

        let root_node = tree.root_node();
        let mut classes = Vec::new();

        // Query to find all class definitions
        static CLASS_QUERY: OnceLock<Query> = OnceLock::new();
        let query = CLASS_QUERY.get_or_init(|| {
            Query::new(tree_sitter_python::language(), CLASS_QUERY_STR)
                .expect("Static class query is invalid")
        });

        // Query to find properties in __init__
        static PROP_QUERY: OnceLock<Query> = OnceLock::new();
        let prop_query = PROP_QUERY.get_or_init(|| {
            Query::new(tree_sitter_python::language(), PROP_QUERY_STR)
                .expect("Static property query is invalid")
        });

        let mut query_cursor = QueryCursor::new();
        let matches = query_cursor.matches(query, root_node, content.as_bytes());

        for m in matches {
            let class_node = m.captures[0].node;
            
            // Extract Full Name (Namespace Aware)
            let mut name_parts = Vec::new();
            let mut curr = Some(class_node);
            while let Some(n) = curr {
                if n.kind() == "class_definition" {
                    if let Some(name_node) = n.child_by_field_name("name") {
                        name_parts.push(get_node_text(name_node, content));
                    }
                }
                curr = n.parent();
            }
            name_parts.reverse();
            let full_name = name_parts.join(".");

            // Extract Parents (Superclasses)
            let mut parents = Vec::new();
            if let Some(superclasses_node) = class_node.child_by_field_name("superclasses") {
                let mut cursor = superclasses_node.walk();
                for child in superclasses_node.children(&mut cursor) {
                    if child.kind() == "identifier" || child.kind() == "attribute" || child.kind() == "subscript" {
                         parents.push(get_node_text(child, content));
                    }
                }
            }

            let mut methods = Vec::new();
            let mut properties = Vec::new();
            let mut relationships = Vec::new();

            // 1. Relationships from inheritance
            for parent in &parents {
                relationships.push(Relationship {
                    target: parent.clone(),
                    rel_type: RelationshipType::Inheritance,
                    label: None,
                });
            }

            if let Some(body_node) = class_node.child_by_field_name("body") {
                let mut cursor = body_node.walk();
                for child in body_node.children(&mut cursor) {
                    let func_node = match child.kind() {
                        "function_definition" | "async_function_definition" => Some(child),
                        "decorated_definition" => {
                            child.child_by_field_name("definition")
                                .filter(|n| n.kind() == "function_definition" || n.kind() == "async_function_definition")
                        }
                        _ => None,
                    };

                    if let Some(fn_node) = func_node {
                        if let Some(func_name_node) = fn_node.child_by_field_name("name") {
                            let method_name = get_node_text(func_name_node, content);
                            let visibility = get_python_visibility(&method_name);

                            // Parameters (for Aggregation/Dependency)
                            if let Some(params_node) = fn_node.child_by_field_name("parameters") {
                                let mut p_cursor = params_node.walk();
                                for param in params_node.children(&mut p_cursor) {
                                    if param.kind() == "typed_parameter" {
                                        if let Some(type_node) = param.child_by_field_name("type") {
                                            let mut resolved = Vec::new();
                                            resolve_types(type_node, content, &mut resolved);
                                            for t in resolved {
                                                let rel_type = if method_name == "__init__" {
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

                            // Return type (for Dependency)
                            if let Some(ret_type_node) = fn_node.child_by_field_name("return_type") {
                                let mut resolved = Vec::new();
                                resolve_types(ret_type_node, content, &mut resolved);
                                for t in resolved {
                                    relationships.push(Relationship {
                                        target: t,
                                        rel_type: RelationshipType::Dependency,
                                        label: None,
                                    });
                                }
                            }

                            // Check for __init__ to extract properties and their types
                            if method_name == "__init__" {
                                let mut prop_cursor = QueryCursor::new();
                                let prop_matches = prop_cursor.matches(prop_query, fn_node, content.as_bytes());
                                
                                for pm in prop_matches {
                                    let obj_node = pm.captures[0].node;
                                    let attr_node = pm.captures[1].node;
                                    
                                    let obj_name = get_node_text(obj_node, content);
                                    let attr_name = get_node_text(attr_node, content);
                                    
                                    if obj_name == "self" {
                                        let prop_visibility = get_python_visibility(&attr_name);
                                        properties.push(PropertyInfo {
                                            name: attr_name.clone(),
                                            visibility: prop_visibility,
                                        });

                                        // Try to find type hint for this property
                                        let mut parent = obj_node.parent();
                                        while let Some(p) = parent {
                                            if p.kind() == "assignment" {
                                                if let Some(type_node) = p.child_by_field_name("type") {
                                                    let mut resolved = Vec::new();
                                                    resolve_types(type_node, content, &mut resolved);
                                                    for t in resolved {
                                                        relationships.push(Relationship {
                                                            target: t,
                                                            rel_type: RelationshipType::Aggregation,
                                                            label: Some(attr_name.clone()),
                                                        });
                                                    }
                                                }
                                                break;
                                            }
                                            parent = p.parent();
                                        }
                                    }
                                }
                            }

                            // Python specific: special methods are treated as private/hidden usually
                            // but for class diagram we might want to show them if they aren't __init__
                            // Following the rule: only show if not starting with _ (unless requested)
                            methods.push(MethodInfo {
                                name: method_name,
                                visibility,
                            });
                        }
                    }
                }
            }

            classes.push(ClassInfo {
                name: full_name,
                methods,
                properties,
                relationships,
            });
        }

        Ok(classes)
    }
}

fn get_python_visibility(name: &str) -> Visibility {
    if name.starts_with("__") && !name.ends_with("__") {
        Visibility::Private
    } else if name.starts_with('_') && !name.ends_with("__") {
        Visibility::Protected
    } else {
        Visibility::Public
    }
}

fn resolve_types(node: Node, content: &str, types: &mut Vec<String>) {
    match node.kind() {
        "identifier" => {
            let name = get_node_text(node, content);
            let primitives: HashSet<&str> = ["str", "int", "float", "bool", "bytes", "None", "Any", "List", "Dict", "Set", "Optional", "Union", "Tuple"].iter().cloned().collect();
            
            if !primitives.contains(name.as_str()) {
                types.push(name);
            }
        }
        _ => {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                resolve_types(child, content, types);
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

    // Helper for tests to reduce boilerplate
    fn parse(content: &str) -> Result<Vec<ClassInfo>> {
        PythonParser.parse(content)
    }

    #[test]
    fn test_parse_simple_class() -> Result<()> {
        let content = "
class Dog:
    def bark(self):
        pass
    
    def _internal(self):
        pass

    def __private(self):
        pass

    def eat(self):
        pass
";
        let classes = parse(content)?;
        
        assert_eq!(classes.len(), 1);
        let dog = &classes[0];
        assert_eq!(dog.name, "Dog");
        
        let bark = dog.methods.iter().find(|m| m.name == "bark").unwrap();
        assert_eq!(bark.visibility, Visibility::Public);

        let internal = dog.methods.iter().find(|m| m.name == "_internal").unwrap();
        assert_eq!(internal.visibility, Visibility::Protected);

        let private = dog.methods.iter().find(|m| m.name == "__private").unwrap();
        assert_eq!(private.visibility, Visibility::Private);
        
        Ok(())
    }

    #[test]
    fn test_parse_nested_classes() -> Result<()> {
        let content = "
class Outer:
    class Inner:
        def inner_method(self): pass
    def outer_method(self): pass
";
        let classes = parse(content)?;
        assert_eq!(classes.len(), 2);
        
        let names: Vec<String> = classes.iter().map(|c| c.name.clone()).collect();
        assert!(names.contains(&"Outer".to_string()));
        assert!(names.contains(&"Outer.Inner".to_string()));
        
        Ok(())
    }

    #[test]
    fn test_parse_properties() -> Result<()> {
        let content = "
class User:
    def __init__(self, name):
        self.name = name
        self._age = 0
        self.__secret = 'hidden'
";
        let classes = parse(content)?;
        let user = &classes[0];
        
        let name = user.properties.iter().find(|p| p.name == "name").unwrap();
        assert_eq!(name.visibility, Visibility::Public);

        let age = user.properties.iter().find(|p| p.name == "_age").unwrap();
        assert_eq!(age.visibility, Visibility::Protected);

        let secret = user.properties.iter().find(|p| p.name == "__secret").unwrap();
        assert_eq!(secret.visibility, Visibility::Private);
        
        Ok(())
    }

    #[test]
    fn test_parse_multiple_classes() -> Result<()> {
        let content = "
class Cat:
    def meow(self): pass

class Bird:
    def fly(self): pass
";
        let classes = parse(content)?;
        assert_eq!(classes.len(), 2);
        
        assert_eq!(classes[0].name, "Cat");
        assert_eq!(classes[0].methods[0].name, "meow");
        
        assert_eq!(classes[1].name, "Bird");
        assert_eq!(classes[1].methods[0].name, "fly");

        Ok(())
    }

    #[test]
    fn test_parse_decorated_methods() -> Result<()> {
        let content = "
class MathUtils:
    @staticmethod
    def add(a, b):
        return a + b

    @classmethod
    def create(cls):
        pass

    def normal(self):
        pass
";
        let classes = parse(content)?;
        let methods: Vec<_> = classes[0].methods.iter().map(|m| &m.name).collect();
        
        assert!(methods.contains(&&"add".to_string()));
        assert!(methods.contains(&&"create".to_string()));
        assert!(methods.contains(&&"normal".to_string()));
        
        Ok(())
    }

    #[test]
    fn test_parse_async_methods() -> Result<()> {
        let content = "
class AsyncService:
    async def fetch_data(self):
        pass

    @log_it
    async def process_data(self):
        pass
";
        let classes = parse(content)?;
        let methods: Vec<_> = classes[0].methods.iter().map(|m| &m.name).collect();
        
        assert!(methods.contains(&&"fetch_data".to_string()));
        assert!(methods.contains(&&"process_data".to_string()));
        
        Ok(())
    }

    #[test]
    fn test_parse_empty_class() -> Result<()> {
        let content = "class Empty: pass";
        let classes = parse(content)?;
        assert_eq!(classes.len(), 1);
        assert!(classes[0].methods.is_empty());
        Ok(())
    }

    #[test]
    fn test_parse_no_classes() -> Result<()> {
        let content = "def standalone_func(): pass";
        let classes = parse(content)?;
        assert!(classes.is_empty());
        Ok(())
    }

    #[test]
    fn test_parse_relationships() -> Result<()> {
        let content = "
class Engine: pass
class Car:
    def __init__(self, engine: Engine):
        self.engine: Engine = engine
        self.driver: Optional[User] = None

    def drive(self, destination: str) -> bool:
        return True

    def repair(self, mechanic: Human):
        pass
";
        let classes = parse(content)?;
        let car = classes.iter().find(|c| c.name == "Car").unwrap();
        
        let rels = &car.relationships;
        
        assert!(rels.iter().any(|r| r.target == "Engine" && r.rel_type == RelationshipType::Aggregation));
        assert!(rels.iter().any(|r| r.target == "User" && r.rel_type == RelationshipType::Aggregation));
        assert!(rels.iter().any(|r| r.target == "Human" && r.rel_type == RelationshipType::Dependency));
        assert!(!rels.iter().any(|r| r.target == "str"));
        assert!(!rels.iter().any(|r| r.target == "bool"));
        
        Ok(())
    }

    #[test]
    fn test_parse_inheritance() -> Result<()> {
        let content = "
class Animal: pass
class Dog(Animal): pass
class Mixed(Animal, Runnable): pass
class Generic(List[int]): pass
";
        let classes = parse(content)?;
        
        let dog = classes.iter().find(|c| c.name == "Dog").unwrap();
        assert!(dog.relationships.iter().any(|r| r.target == "Animal" && r.rel_type == RelationshipType::Inheritance));

        let mixed = classes.iter().find(|c| c.name == "Mixed").unwrap();
        assert!(mixed.relationships.iter().any(|r| r.target == "Animal" && r.rel_type == RelationshipType::Inheritance));
        assert!(mixed.relationships.iter().any(|r| r.target == "Runnable" && r.rel_type == RelationshipType::Inheritance));

        let generic = classes.iter().find(|c| c.name == "Generic").unwrap();
        assert!(generic.relationships.iter().any(|r| r.target == "List[int]" && r.rel_type == RelationshipType::Inheritance));

        Ok(())
    }
}
