use std::sync::OnceLock;
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
    static QUERY: OnceLock<Query> = OnceLock::new();
    let query = QUERY.get_or_init(|| {
        Query::new(language, "(class_definition) @class")
            .expect("Failed to create Tree-sitter query")
    });

    let mut query_cursor = QueryCursor::new();
    let matches = query_cursor.matches(query, root_node, content.as_bytes());

    for m in matches {
        let class_node = m.captures[0].node;
        
        // Extract Class Name
        let name = class_node.child_by_field_name("name")
            .map(|n| get_node_text(n, content))
            .unwrap_or_else(|| "Anonymous".to_string());

        // Extract Methods (Direct children of the body block)
        let mut methods = Vec::new();
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
                        if !method_name.starts_with('_') {
                            methods.push(method_name);
                        }
                    }
                }
            }
        }

        classes.push(ClassInfo {
            name,
            methods,
            properties: Vec::new(), // TODO: Implement property extraction
            parents: Vec::new(),    // TODO: Implement inheritance extraction
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
    async fn fetch_data(self):
        pass

    @log_it
    async def process_data(self):
        pass
";
        let classes = parse_python_file(content)?;
        assert_eq!(classes.len(), 1);
        let methods = &classes[0].methods;
        
        // Note: my sample had 'async fn' but it should be 'async def'
        // Actually I'll fix the sample to valid python
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
}
