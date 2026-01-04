use tree_sitter::{Parser, Query, QueryCursor, Node};
use crate::models::{ClassInfo, Relationship, RelationshipType, Visibility, MethodInfo, PropertyInfo};
use anyhow::{Result, Context};
use std::collections::HashSet;
use super::LanguageParser;

pub struct RubyParser;

impl LanguageParser for RubyParser {
    fn extensions(&self) -> &[&str] {
        &["rb"]
    }

    fn parse(&self, content: &str) -> Result<Vec<ClassInfo>> {
        let mut parser = Parser::new();
        let language = tree_sitter_ruby::language();
        parser.set_language(language)
            .context("Error loading Ruby grammar")?;

        let tree = parser.parse(content, None)
            .context("Failed to parse Ruby content")?;

        let root_node = tree.root_node();
        let mut classes = Vec::new();

        // 1. Find all classes and modules
        let query_str = "
            [(class) (module)] @entity
        ";
        let query = Query::new(language, query_str).expect("Invalid Ruby entity query");
        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(&query, root_node, content.as_bytes());

        for m in matches {
            let entity_node = m.captures[0].node;
            
            // Extract Full Name (Namespace Aware)
            let mut name_parts = Vec::new();
            let mut curr = Some(entity_node);
            while let Some(n) = curr {
                if n.kind() == "class" || n.kind() == "module" {
                    if let Some(name_node) = n.child_by_field_name("name") {
                        name_parts.push(get_node_text(name_node, content));
                    }
                }
                curr = n.parent();
            }
            name_parts.reverse();
            let full_name = name_parts.join("::");

            // Extract Superclass
            let mut superclass = None;
            if entity_node.kind() == "class" {
                if let Some(super_node) = entity_node.child_by_field_name("superclass") {
                    if let Some(const_node) = super_node.child(1) { // class < Super (index 1 is the superclass)
                         superclass = Some(get_node_text(const_node, content));
                    }
                }
            }

            let mut methods = Vec::new();
            let mut properties = Vec::new();
            let mut relationships = Vec::new();

            if let Some(target) = superclass {
                relationships.push(Relationship {
                    target,
                    rel_type: RelationshipType::Inheritance,
                    label: None,
                });
            }

            // Process body with visibility tracking
            if let Some(body) = entity_node.child_by_field_name("body") {
                let mut current_visibility = Visibility::Public;
                let mut body_cursor = body.walk();
                for child in body.children(&mut body_cursor) {
                    match child.kind() {
                        "method" => {
                            if let Some(name_node) = child.child_by_field_name("name") {
                                let m_name = get_node_text(name_node, content);
                                
                                methods.push(MethodInfo {
                                    name: m_name.clone(),
                                    visibility: current_visibility,
                                });
                                
                                // Heuristic: Check parameters for relationships
                                if let Some(params) = child.child_by_field_name("parameters") {
                                    let mut p_cursor = params.walk();
                                    for param in params.children(&mut p_cursor) {
                                        if param.kind() == "identifier" {
                                            let p_text = get_node_text(param, content);
                                            
                                            // A simple blocklist to avoid creating relationships for common non-class parameter names.
                                            const IGNORED_PARAMS: &[&str] = &["name", "age", "id", "count", "size", "length", "width", "height", "index", "key", "value", "message", "text"];

                                            if !IGNORED_PARAMS.contains(&p_text.as_str()) {
                                                let target = to_pascal_case(&p_text);
                                                
                                                if !is_ruby_builtin(&target) {
                                                    let rel_type = if m_name == "initialize" {
                                                        RelationshipType::Aggregation
                                                    } else {
                                                        RelationshipType::Dependency
                                                    };
                                                    relationships.push(Relationship {
                                                        target,
                                                        rel_type,
                                                        label: Some(p_text.clone()),
                                                    });
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        "singleton_method" => {
                            if let Some(name_node) = child.child_by_field_name("name") {
                                methods.push(MethodInfo {
                                    name: format!("self.{}", get_node_text(name_node, content)),
                                    visibility: Visibility::Public,
                                });
                            }
                        }
                        "call" | "command" | "identifier" => {
                            let cmd = if child.kind() == "identifier" {
                                get_node_text(child, content)
                            } else {
                                child.child_by_field_name("method")
                                    .map(|n| get_node_text(n, content))
                                    .unwrap_or_default()
                            };

                            match cmd.as_str() {
                                "private" | "protected" | "public" => {
                                    let has_args = child.child_by_field_name("arguments").is_some();
                                    if !has_args {
                                        current_visibility = match cmd.as_str() {
                                            "private" => Visibility::Private,
                                            "protected" => Visibility::Protected,
                                            _ => Visibility::Public,
                                        };
                                    }
                                }
                                "attr_accessor" | "attr_reader" | "attr_writer" => {
                                    if let Some(args) = child.child_by_field_name("arguments") {
                                        let mut arg_cursor = args.walk();
                                        for arg in args.children(&mut arg_cursor) {
                                            let arg_text = get_node_text(arg, content);
                                            properties.push(PropertyInfo {
                                                name: arg_text.trim_start_matches(':').to_string(),
                                                visibility: current_visibility,
                                            });
                                        }
                                    }
                                }
                                "include" | "extend" | "prepend" => {
                                    if let Some(args) = child.child_by_field_name("arguments") {
                                        let mut arg_cursor = args.walk();
                                        for arg in args.children(&mut arg_cursor) {
                                            let arg_text = get_node_text(arg, content);
                                            relationships.push(Relationship {
                                                target: arg_text,
                                                rel_type: RelationshipType::Dependency,
                                                label: Some(cmd.clone()),
                                            });
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                        _ => {}
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

fn is_ruby_builtin(name: &str) -> bool {
    let builtins: HashSet<&str> = [
        "String", "Integer", "Float", "Array", "Hash", "Symbol", "TrueClass", "FalseClass", "NilClass",
        "Object", "Kernel", "Module", "Class", "Numeric", "Range", "Regexp", "Proc", "Method", "IO", "File", "Dir", "Time"
    ].iter().cloned().collect();
    builtins.contains(name) || name == "Data" || name == "Arg"
}

fn get_node_text(node: Node, content: &str) -> String {
    node.utf8_text(content.as_bytes())
        .map(str::to_string)
        .unwrap_or_default()
}

fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(content: &str) -> Result<Vec<ClassInfo>> {
        RubyParser.parse(content)
    }

    #[test]
    fn test_parse_simple_class() -> Result<()> {
        let content = "
class Dog
  def bark
    puts 'Woof!'
  end

  def eat(food)
  end

  private
  def sleep
  end
end
";
        let classes = parse(content)?;
        assert_eq!(classes.len(), 1);
        let dog = &classes[0];
        assert_eq!(dog.name, "Dog");
        
        let bark = dog.methods.iter().find(|m| m.name == "bark").unwrap();
        assert_eq!(bark.visibility, Visibility::Public);

        let sleep = dog.methods.iter().find(|m| m.name == "sleep").unwrap();
        assert_eq!(sleep.visibility, Visibility::Private);
        Ok(())
    }

    #[test]
    fn test_ruby_namespace() -> Result<()> {
        let content = "
module UI
  class Button; end
end
";
        let classes = parse(content)?;
        let button = classes.iter().find(|c| c.name.contains("Button")).unwrap();
        assert_eq!(button.name, "UI::Button");
        Ok(())
    }

    #[test]
    fn test_parse_inheritance() -> Result<()> {
        let content = "
class Animal
end

class Cat < Animal
end
";
        let classes = parse(content)?;
        let cat = classes.iter().find(|c| c.name == "Cat").unwrap();
        assert!(cat.relationships.iter().any(|r| 
            r.target == "Animal" && r.rel_type == RelationshipType::Inheritance
        ));
        Ok(())
    }

    #[test]
    fn test_parse_modules_and_mixins() -> Result<()> {
        let content = "
module Swimmable
  def swim; end
end

class Fish
  include Swimmable
  extend Flyable
  prepend Breathable
end
";
        let classes = parse(content)?;
        
        // Modules should be treated as classes for the sake of the diagram
        let swimmable = classes.iter().find(|c| c.name == "Swimmable").expect("Should find Swimmable module");
        assert!(swimmable.methods.iter().any(|m| m.name == "swim"));

        let fish = classes.iter().find(|c| c.name == "Fish").unwrap();
        // Mixins are often represented as a form of inheritance/realization in UML
        assert!(fish.relationships.iter().any(|r| r.target == "Swimmable" && r.label.as_deref() == Some("include")));
        assert!(fish.relationships.iter().any(|r| r.target == "Flyable" && r.label.as_deref() == Some("extend")));
        assert!(fish.relationships.iter().any(|r| r.target == "Breathable" && r.label.as_deref() == Some("prepend")));
        
        Ok(())
    }

    #[test]
    fn test_parse_attributes() -> Result<()> {
        let content = "
class User
  attr_accessor :name, :email
  attr_reader :id
  attr_writer :password

  def initialize(name)
    @name = name
  end
end
";
        let classes = parse(content)?;
        let user = &classes[0];
        
        assert!(user.properties.iter().any(|p| p.name == "name"));
        assert!(user.properties.iter().any(|p| p.name == "email"));
        assert!(user.properties.iter().any(|p| p.name == "id"));
        assert!(user.properties.iter().any(|p| p.name == "password"));
        
        Ok(())
    }

    #[test]
    fn test_parse_relationships_aggregation() -> Result<()> {
        let content = "
class Engine
end

class Car
  def initialize(engine)
    @engine = engine
  end
end
";
        let classes = parse(content)?;
        let car = classes.iter().find(|c| c.name == "Car").unwrap();
        
        assert!(car.relationships.iter().any(|r| 
            r.target == "Engine" && r.rel_type == RelationshipType::Aggregation
        ));
        Ok(())
    }

    #[test]
    fn test_parse_multiple_entities() -> Result<()> {
        let content = "
class A; end
class B; end
module M; end
";
        let classes = parse(content)?;
        assert_eq!(classes.len(), 3);
        let names: Vec<_> = classes.iter().map(|c| &c.name).collect();
        assert!(names.contains(&&"A".to_string()));
        assert!(names.contains(&&"B".to_string()));
        assert!(names.contains(&&"M".to_string()));
        Ok(())
    }

    #[test]
    fn test_parse_complex_relationships() -> Result<()> {
        let content = "
class Processor
  def process(data_source)
    @source = data_source
  end

  def self.create(config)
    new(config)
  end
end
";
        let classes = parse(content)?;
        let processor = &classes[0];
        
        assert!(processor.relationships.iter().any(|r| 
            r.target == "DataSource" && r.rel_type == RelationshipType::Dependency
        ));
        Ok(())
    }
}
