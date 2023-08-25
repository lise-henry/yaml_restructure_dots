// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::error;

use serde_yaml::Value;

#[derive(Debug, PartialEq)]
pub enum ValueType {
    Null,
    Bool,
    Number,
    String,
    List,
    Mapping,
    Tagged,
}

const INDENT: &'static str = "    ";
const DESCRIPTION: &'static str = "__description__";

fn indent(content: &mut String, n: u8) {
    for _ in 0..n {
        content.push_str(INDENT);
    }
}

fn document_val(content: &mut String, val: &Value, description: Option<&Value>, mut indent_level: u8) -> error::Result<()> {
    match val {
        Value::Mapping(ref m) => {
            if indent_level > 0 {
                content.push_str("\n");
            }
            for (key, value) in m.iter() {
                let ty = match value {
                    Value::Null => ValueType::Null,
                    Value::Bool(_) => ValueType::Bool,
                    Value::Number(_) => ValueType::Number,
                    Value::String(_) => ValueType::String,
                    Value::Sequence(_) => ValueType::List,
                    Value::Mapping(_) => ValueType::Mapping,
                    Value::Tagged(_) => ValueType::Tagged,
                };
                // Try displaying the description, if it exists
                let desc_value = description.and_then(|d| d.as_mapping())
                    .and_then(|m| m.get(key));
                if let Some(inner) = desc_value {
                    match inner {
                        Value::String(s) => {
                            // Found a description, displays it
                            indent(content, indent_level);
                            content.push_str("# ");
                            content.push_str(s);
                            content.push_str("\n");
                        }
                        Value::Mapping(m) => {
                            // Try to see if there is a description field for this mapping
                            let desc = m.get(DESCRIPTION)
                                .and_then(|v| v.as_str());
                            if let Some(s) = desc {
                                indent(content, indent_level);
                                content.push_str("# ");
                                content.push_str(s);
                                content.push_str("\n");
                            }
                        }
                        _ => {

                        }
                    }
                }
                
                
                // Display the key name
                for _ in 0..indent_level {
                    content.push_str(INDENT);
                }
                let k = if key.is_string() {
                    key.as_str().unwrap().to_owned()
                } else {
                    format!("{:?}", key)
                };
                content.push_str(&format!("{} ({:?}): ", &k, ty));
                document_val(content, value, desc_value, indent_level + 1);
            }
        },
        Value::Sequence(ref s) => {
            content.push_str("\n");
            for v in s.iter() {
                indent(content, indent_level);
                content.push_str("- ");
                document_val(content, v, None, indent_level + 1);
            }
        }
        _ => {
            content.push_str(&serde_yaml::to_string(val)?);
        },
    }
    Ok(())    
}


/// Uses a YAML representation from a default struct to document an API or options
/// in a YAML-looking way
///
/// The idea is to first generate a yaml representation with YourStruct::default()
/// to get a mostly automated API description.
///
/// # Example
///
/// ```
///         let desc_yaml = r#"
/// foo:
///     __description__: Description for foo
///     bar: Description for bar
/// "#;
/// 
///         let yaml = r#"
/// foo:
///     bar: 42
/// "#;
/// 
///         let expected = "# Description for foo
/// foo (Mapping): \n    # Description for bar
///     bar (Number): 42
/// ";
///         let value: serde_yaml::Value = serde_yaml::from_str(&yaml).unwrap();
///         let desc: serde_yaml::Value = serde_yaml::from_str(&desc_yaml).unwrap();
///         let s = yaml_extras::document(&value, Some(&desc)).unwrap();
///         assert_eq!(s, expected);
/// ```
pub fn document(val: &Value, description: Option<&Value>) -> error::Result<String> {
    let mut content = String::new();

    document_val(&mut content, val, description, 0)?;
    
    Ok(content)
}


#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::{assert_eq, assert_ne};
    use serde_yaml::Value;

    #[test]
    fn document_simple() {
        let desc_yaml = r#"
foo:
    __description__: Description for foo
    bar: Description for bar
"#;

        let yaml = r#"
foo:
    bar: 42
"#;

        let expected = r#"# Description for foo
foo (Mapping): 
    # Description for bar
    bar (Number): 42
"#;
        let value: Value = serde_yaml::from_str(&yaml).unwrap();
        let desc: Value = serde_yaml::from_str(&desc_yaml).unwrap();
        let s = document(&value, Some(&desc)).unwrap();
        assert_eq!(s, expected);
    }
}
