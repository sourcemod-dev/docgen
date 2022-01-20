use serde::{Deserialize, Serialize};

use std::fmt::{Display, Formatter, Result};

pub mod argument;
pub mod base;
pub mod constant;
pub mod define;
pub mod enum_struct;
pub mod enumeration;
pub mod function;
pub mod method_map;
pub mod property;
pub mod type_def;
pub mod type_set;

pub use self::argument::Argument;
pub use self::base::{Declaration, DocLocation, Documentation};
pub use self::constant::Constant;
pub use self::define::Define;
pub use self::enum_struct::{DPEnumStruct, EnumStruct, Field};
pub use self::enumeration::{DPEnumeration, Entry, Enumeration};
pub use self::function::Function;
pub use self::method_map::{DPMethodMap, MethodMap};
pub use self::property::Property;
pub use self::type_def::{parse_type_signature, TypeDefinition, TypeSignature};
pub use self::type_set::{DPTypeSet, Type, TypeSet};

#[derive(Debug, Serialize, Deserialize)]
pub enum SymbolType {
    Function,
    MethodMap,
    Property,
    Constant,
    Define,
    Enum,
    EnumStruct,
    Field,
    TypeSet,
    TypeDefinition,
}

impl Display for SymbolType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match *self {
            Self::Function => write!(f, "function"),
            Self::MethodMap => write!(f, "methodmap"),
            Self::Property => write!(f, "property"),
            Self::Constant => write!(f, "constant"),
            Self::Define => write!(f, "define"),
            Self::Enum => write!(f, "enumeration"),
            Self::EnumStruct => write!(f, "enumstruct"),
            Self::Field => write!(f, "field"),
            Self::TypeSet => write!(f, "typeset"),
            Self::TypeDefinition => write!(f, "typedef"),
        }
    }
}
