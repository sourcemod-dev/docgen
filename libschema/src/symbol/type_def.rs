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

pub fn parse_type_signature(s: &str) -> TypeSignature {
    let sig = &s[9..];

    let param_start = sig.find('(').unwrap();

    let return_type = &sig[..param_start as usize];

    let mut param_section = &sig[param_start as usize..];
    param_section = &param_section[1..param_section.len() - 1];

    TypeSignature {
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
    }
}
