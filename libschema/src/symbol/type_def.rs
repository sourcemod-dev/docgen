use std::ops::ShlAssign;

use serde::{Deserialize, Serialize};

use crate::metadata::Metadata;
use crate::symbol::{Argument, Declaration, Metable};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeDefinition {
    #[serde(flatten)]
    pub declaration: Declaration,

    /// Function signature
    pub r#type: String,

    /// Parsed function signature
    pub parsed_signature: Option<TypeSignature>,
}

impl Metable for TypeDefinition {
    fn metadata(&mut self) -> &mut Option<Metadata> {
        &mut self.declaration.documentation.metadata
    }
}

impl ShlAssign for TypeDefinition {
    fn shl_assign(&mut self, rhs: Self) {
        self.declaration <<= rhs.declaration;
        self.r#type = rhs.r#type;
        self.parsed_signature = rhs.parsed_signature;
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeSignature {
    /// Return type of the function
    pub return_type: String,

    /// Arguments of the function
    pub arguments: Vec<Argument>,
}

/// Parses a `function <return>(<args>)` signature as rendered in `type`.
///
/// Returns `None` for anything that isn't a function signature,
/// such as `typedef Address = int64;`.
pub fn parse_type_signature(s: &str) -> Option<TypeSignature> {
    let sig = s.strip_prefix("function ")?;

    let param_start = sig.find('(')?;
    let param_end = sig.rfind(')')?;

    if param_end < param_start {
        return None;
    }

    let return_type = &sig[..param_start];

    let param_section = &sig[param_start + 1..param_end];

    Some(TypeSignature {
        return_type: return_type.to_string(),
        arguments: {
            let mut args = Vec::new();

            param_section
                .split(", ")
                .map(|v| v.split(' '))
                .for_each(|v| {
                    let parts = v.collect::<Vec<_>>();

                    match parts.len() {
                        2 => {
                            args.push(Argument {
                                r#type: parts[0].to_string(),
                                name: parts[1].to_string(),
                                decl: parts.join(" "),
                                default: None,
                            });
                        }
                        l if l > 2 => {
                            args.push(Argument {
                                r#type: parts[..2].join(" "),
                                name: parts[2].to_string(),
                                decl: parts.join(" "),
                                default: None,
                            });
                        }
                        _ => (),
                    }
                });

            args
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_function_signatures() {
        let sig = parse_type_signature("function Action(Handle timer, const char[] name)").unwrap();
        assert_eq!(sig.return_type, "Action");
        assert_eq!(sig.arguments.len(), 2);
        assert_eq!(sig.arguments[1].r#type, "const char[]");
        assert_eq!(sig.arguments[1].name, "name");

        assert!(parse_type_signature("function void()").unwrap().arguments.is_empty());
    }

    #[test]
    fn ignores_non_function_types() {
        assert_eq!(parse_type_signature("int64"), None);
        assert_eq!(parse_type_signature(""), None);
    }
}
