use crate::models::ClassInfo;
use std::fmt::Write;

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

    // 2. Define Relationships (Inheritance)
    for class in classes {
        for parent in &class.parents {
            // Parent <|-- Child
            writeln!(&mut diagram, "    {} <|-- {}", parent, class.name).unwrap();
        }
    }

    diagram
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_mermaid_simple() {
        let classes = vec![
            ClassInfo {
                name: "Dog".to_string(),
                methods: vec!["bark".to_string()],
                properties: vec!["breed".to_string()],
                parents: vec!["Animal".to_string()],
            },
            ClassInfo {
                name: "Animal".to_string(),
                methods: vec!["eat".to_string()],
                properties: vec![],
                parents: vec![],
            }
        ];

        let output = generate_mermaid(&classes);
        
        let expected = "classDiagram\n    class Dog {\n        +breed\n        +bark()\n    }\n    class Animal {\n        +eat()\n    }\n    Animal <|-- Dog\n";
        assert_eq!(output, expected);
    }
}
