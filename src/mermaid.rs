use crate::models::{ClassInfo, RelationshipType, Visibility};
use std::fmt::Write;
use std::collections::HashSet;

pub fn generate_mermaid(classes: &[ClassInfo], enabled_visibilities: &[Visibility]) -> String {
    let mut diagram = String::new();
    writeln!(&mut diagram, "classDiagram").unwrap();

    // 1. Define Classes
    for class in classes {
        writeln!(&mut diagram, "    class {} {{", class.name).unwrap();
        
        // Properties
        for prop in &class.properties {
            if enabled_visibilities.contains(&prop.visibility) {
                let symbol = match prop.visibility {
                    Visibility::Public => "+",
                    Visibility::Protected => "#",
                    Visibility::Private => "-",
                    Visibility::Internal => "~",
                };
                writeln!(&mut diagram, "        {}{}", symbol, prop.name).unwrap();
            }
        }

        // Methods
        for method in &class.methods {
            if enabled_visibilities.contains(&method.visibility) {
                let symbol = match method.visibility {
                    Visibility::Public => "+",
                    Visibility::Protected => "#",
                    Visibility::Private => "-",
                    Visibility::Internal => "~",
                };
                writeln!(&mut diagram, "        {}{}()", symbol, method.name).unwrap();
            }
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
    use crate::models::{Relationship, RelationshipType, MethodInfo, PropertyInfo};

    #[test]
    fn test_generate_mermaid_complex() {
        let classes = vec![
            ClassInfo {
                name: "Car".to_string(),
                methods: vec![
                    MethodInfo { name: "drive".to_string(), visibility: Visibility::Public },
                    MethodInfo { name: "service".to_string(), visibility: Visibility::Private },
                ],
                properties: vec![
                    PropertyInfo { name: "engine".to_string(), visibility: Visibility::Public },
                ],
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

        let enabled = vec![Visibility::Public];
        let output = generate_mermaid(&classes, &enabled);
        
        assert!(output.contains("+drive()"));
        assert!(!output.contains("-service()"));
        assert!(output.contains("+engine"));
        assert!(output.contains("Engine o-- Car : engine"));
        assert!(output.contains("Vehicle <|-- Car"));
    }
}