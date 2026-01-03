use tree_sitter::{Parser, Query, QueryCursor, Node};
use crate::models::ClassInfo;
use anyhow::{Result, Context};

pub fn parse_python_file(content: &str) -> Result<Vec<ClassInfo>> {
    let mut parser = Parser::new();
    let language = tree_sitter_python::language();
    parser.set_language(language)
        .context("Error loading Python grammar")?;

    let tree = parser.parse(content, None)
        .context("Failed to parse Python content")?;

    let root_node = tree.root_node();
    let mut classes = Vec::new();

    // Query to find all class definitions
    let query_str = "(class_definition) @class";
    let query = Query::new(language, query_str)
        .context("Failed to create Tree-sitter query")?;

    // Query to find properties in __init__
    let prop_query_str = "
        (assignment left: (attribute object: (identifier) @obj attribute: (identifier) @attr))
        (assignment left: (pattern_list (attribute object: (identifier) @obj attribute: (identifier) @attr)))
    ";
    let prop_query = Query::new(language, prop_query_str)
        .context("Failed to create property query")?;

    let mut query_cursor = QueryCursor::new();
    let matches = query_cursor.matches(&query, root_node, content.as_bytes());

    for m in matches {
        let class_node = m.captures[0].node;
        
        // Extract Class Name
        let name = class_node.child_by_field_name("name")
            .map(|n| get_node_text(n, content))
            .unwrap_or_else(|| "Anonymous".to_string());

        // Extract Methods and Properties
        let mut methods = Vec::new();
        let mut properties = Vec::new();

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
                        
                        // Check for __init__ to extract properties
                        if method_name == "__init__" {
                            let mut prop_cursor = QueryCursor::new();
                            let prop_matches = prop_cursor.matches(&prop_query, fn_node, content.as_bytes());
                            
                            for pm in prop_matches {
                                let obj_node = pm.captures[0].node;
                                let attr_node = pm.captures[1].node;
                                
                                let obj_name = get_node_text(obj_node, content);
                                let attr_name = get_node_text(attr_node, content);
                                
                                if obj_name == "self" && !attr_name.starts_with('_') {
                                    properties.push(attr_name);
                                }
                            }
                        }

                        if !method_name.starts_with('_') {
                            methods.push(method_name);
                        }
                    }
                }
            }
        }

        // Extract Parents (Superclasses)
        let mut parents = Vec::new();
        if let Some(superclasses_node) = class_node.child_by_field_name("superclasses") {
            let mut cursor = superclasses_node.walk();
            for child in superclasses_node.children(&mut cursor) {
                // We typically want identifiers, attributes (module.Class), or calls (Generic[T])
                // Simple case: ignore punctuation like "(" , ")"
                if child.kind() == "identifier" || child.kind() == "attribute" || child.kind() == "subscript" {
                     parents.push(get_node_text(child, content));
                }
            }
        }

        classes.push(ClassInfo {
            name,
            methods,
            properties,
            parents,
        });
    }

    Ok(classes)
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
    fn test_parse_simple_class() -> Result<()> {
        let content = "
class Dog:
    def bark(self):
        pass
    
    def _internal(self):
        pass

    def eat(self):
        pass
";
        let classes = parse_python_file(content)?;
        
        assert_eq!(classes.len(), 1);
        let dog = &classes[0];
        assert_eq!(dog.name, "Dog");
        assert_eq!(dog.methods, vec!["bark", "eat"]);
        // Should NOT include _internal
        
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
        let classes = parse_python_file(content)?;
        assert_eq!(classes.len(), 2);
        
        assert_eq!(classes[0].name, "Cat");
        assert_eq!(classes[0].methods, vec!["meow"]);
        
        assert_eq!(classes[1].name, "Bird");
        assert_eq!(classes[1].methods, vec!["fly"]);

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
        let classes = parse_python_file(content)?;
        let methods = &classes[0].methods;
        
        assert!(methods.contains(&"add".to_string()), "Should find @staticmethod 'add'");
        assert!(methods.contains(&"create".to_string()), "Should find @classmethod 'create'");
        assert!(methods.contains(&"normal".to_string()), "Should find normal method");
        
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
        let classes = parse_python_file(content)?;
        let methods = &classes[0].methods;
        
        assert!(methods.contains(&"fetch_data".to_string()));
        assert!(methods.contains(&"process_data".to_string()));
        
        Ok(())
    }

    #[test]
    fn test_parse_empty_class() -> Result<()> {
        let content = "class Empty: pass";
        let classes = parse_python_file(content)?;
        assert_eq!(classes.len(), 1);
        assert!(classes[0].methods.is_empty());
        Ok(())
    }

    #[test]
    fn test_parse_no_classes() -> Result<()> {
        let content = "def standalone_func(): pass";
        let classes = parse_python_file(content)?;
        assert!(classes.is_empty());
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
        let classes = parse_python_file(content)?;
        // Current query finds all class definitions
        assert_eq!(classes.len(), 2);
        
        let names: Vec<String> = classes.iter().map(|c| c.name.clone()).collect();
        assert!(names.contains(&"Outer".to_string()));
        assert!(names.contains(&"Inner".to_string()));
        
        Ok(())
    }

    #[test]
    fn test_parse_complex_properties() -> Result<()> {
        let content = "
class ComplexUser:
    def __init__(self):
        self.name: str = 'Named'
        self.x, self.y = 0, 0
";
        let classes = parse_python_file(content)?;
        let props = &classes[0].properties;
        
        assert!(props.contains(&"name".to_string()), "Should support type hints");
        assert!(props.contains(&"x".to_string()), "Should support tuple assignment x");
        assert!(props.contains(&"y".to_string()), "Should support tuple assignment y");
        
        Ok(())
    }

    #[test]
    fn test_parse_properties() -> Result<()> {
        let content = "
class User:
    def __init__(self, name):
        self.name = name
        self.age = 0
        self._private = 'hidden'

    def other(self):
        self.dynamic = 1 # Should ideally be ignored if we only look at __init__
";
        let classes = parse_python_file(content)?;
        let user = &classes[0];
        
        assert!(user.properties.contains(&"name".to_string()), "Should find 'name' property");
        assert!(user.properties.contains(&"age".to_string()), "Should find 'age' property");
        assert!(!user.properties.contains(&"_private".to_string()), "Should ignore private properties");
        
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
        let classes = parse_python_file(content)?;
        
        // Find Dog
        let dog = classes.iter().find(|c| c.name == "Dog").unwrap();
        assert_eq!(dog.parents, vec!["Animal"]);

        // Find Mixed
        let mixed = classes.iter().find(|c| c.name == "Mixed").unwrap();
        assert!(mixed.parents.contains(&"Animal".to_string()));
        assert!(mixed.parents.contains(&"Runnable".to_string()));

        // Find Generic
        let generic = classes.iter().find(|c| c.name == "Generic").unwrap();
        // The simple extractor might get "List[int]" or "List" depending on logic. 
        // My implementation checks for "subscript" kind too, so it should grab the whole node text "List[int]"
        // or if it treats subscript node as one child.
        assert_eq!(generic.parents[0], "List[int]");

        Ok(())
    }
}
