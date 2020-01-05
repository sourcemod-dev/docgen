use serde::{Serialize, Deserialize};

use crate::symbol::{
    Declaration,
    Argument,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeDefinition {
    #[serde(flatten)]
    pub declaration: Declaration,

    /// Function signature
    pub r#type: String,

    /// Parsed function signature
    pub parsed_signature: Option<TypeSignature>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeSignature{
    /// Return type of the function
    pub return_type: String,

    /// Arguments of the function
    pub arguments: Vec<Argument>,
}

pub fn parse_type_signature(s: &str) -> TypeSignature {
    let sig = &s[9..];

    let param_start = sig.find('(').unwrap();

    let return_type = &sig[..param_start as usize];

    let mut param_section = &sig[param_start as usize..];
    param_section = &param_section[1..param_section.len() - 1];

    TypeSignature {
        return_type: return_type.to_string(),
        arguments: {
            if param_section.find(',').is_none() {
                Vec::new()
            } else {
                param_section
                    .split(", ")
                    .map(|v| v.split(" "))
                    .map(|v| {
                        let parts = v.collect::<Vec<_>>();

                        if parts.len() == 2 {
                            return Argument {
                                r#type: parts[0].to_string(),
                                name: parts[1].to_string(),
                                decl: parts.join(" "),
                                default: None,
                            };
                        }

                        return Argument {
                            r#type: parts[..2].join(" "),
                            name: parts[2].to_string(),
                            decl: parts.join(" "),
                            default: None,
                        }
                    })
                    .collect()
            }
        },
    }
}
