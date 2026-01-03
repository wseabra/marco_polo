use crate::models::{ClassInfo, RelationshipType};
use std::fmt::Write;
use std::collections::HashSet;

pub fn generate_mermaid(classes: &[ClassInfo]) -> String {
    let mut diagram = String::new();
    writeln!(&mut diagram, "classDiagram").unwrap();

    // 1. Define Classes
    for class in classes {
        writeln!(&mut diagram, "    class {} {{", class.name).unwrap();
        
        // Properties
        for prop in &class.properties {
            writeln!(&mut diagram, "        +{}", prop).unwrap();
        }

        // Methods
        for method in &class.methods {
            writeln!(&mut diagram, "        +{}()", method).unwrap();
        }

        writeln!(&mut diagram, "    }}").unwrap();
    }

    // 2. Define Relationships
    let mut seen = HashSet::new();
    for class in classes {
        for rel in &class.relationships {
            let arrow = match rel.rel_type {
                RelationshipType::Inheritance => "<|--",
                RelationshipType::Composition => "*--",
                RelationshipType::Aggregation => "o--",
                RelationshipType::Dependency => "..>",
            };

            let line = if let Some(label) = &rel.label {
                format!("    {} {} {} : {}", rel.target, arrow, class.name, label)
            } else {
                format!("    {} {} {}", rel.target, arrow, class.name)
            };

            if seen.insert(line.clone()) {
                writeln!(&mut diagram, "{}", line).unwrap();
            }
        }
    }

    diagram
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Relationship, RelationshipType};

    #[test]
    fn test_generate_mermaid_complex() {
        let classes = vec![
            ClassInfo {
                name: "Car".to_string(),
                methods: vec![],
                properties: vec![],
                relationships: vec![
                    Relationship {
                        target: "Engine".to_string(),
                        rel_type: RelationshipType::Aggregation,
                        label: Some("engine".to_string()),
                    },
                    Relationship {
                        target: "Vehicle".to_string(),
                        rel_type: RelationshipType::Inheritance,
                        label: None,
                    }
                ],
            },
        ];

        let output = generate_mermaid(&classes);
        assert!(output.contains("Engine o-- Car : engine"));
        assert!(output.contains("Vehicle <|-- Car"));
    }
}
