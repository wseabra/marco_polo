use crate::models::ClassInfo;

pub fn generate_mermaid(classes: &[ClassInfo]) -> String {
    let mut diagram = String::from("classDiagram\n");

    // 1. Define Classes
    for class in classes {
        diagram.push_str(&format!("    class {} {{\n", class.name));
        
        // Properties
        for prop in &class.properties {
            diagram.push_str(&format!("        +{}\n", prop));
        }

        // Methods
        for method in &class.methods {
            diagram.push_str(&format!("        +{}()\n", method));
        }

        diagram.push_str("    }\n");
    }

    // 2. Define Relationships (Inheritance)
    for class in classes {
        for parent in &class.parents {
            // Parent <|-- Child
            diagram.push_str(&format!("    {} <|-- {}\n", parent, class.name));
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
        
        assert!(output.contains("classDiagram"));
        assert!(output.contains("class Dog {"));
        assert!(output.contains("+bark()"));
        assert!(output.contains("+breed"));
        assert!(output.contains("Animal <|-- Dog"));
        assert!(output.contains("class Animal {"));
        assert!(output.contains("+eat()"));
    }
}
