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

    let mut query_cursor = QueryCursor::new();
    let matches = query_cursor.matches(&query, root_node, content.as_bytes());

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
                if child.kind() == "function_definition" {
                    if let Some(func_name_node) = child.child_by_field_name("name") {
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
        .unwrap_or("")
        .to_string()
}