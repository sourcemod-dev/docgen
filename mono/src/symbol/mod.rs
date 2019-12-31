use serde::{Serialize, Deserialize};

use std::fmt::{
    Display,
    Formatter,
    Result
};

pub mod base;
pub mod argument;
pub mod constant;
pub mod enumeration;
pub mod function;
pub mod method_map;
pub mod enum_struct;
pub mod property;
pub mod type_def;
pub mod type_set;

pub use self::enum_struct::{
    EnumStruct,
    Field,
};
pub use self::base::{
    DocLocation,
    Documentation,
    Declaration,
};
pub use self::argument::Argument;
pub use self::constant::Constant;
pub use self::enumeration::Enumeration;
pub use self::function::Function;
pub use self::method_map::MethodMap;
pub use self::property::Property;
pub use self::type_def::TypeDefinition;
pub use self::type_set::TypeSet;

#[derive(Debug, Serialize, Deserialize)]
pub enum SymbolType {
    Function,
    MethodMap,
    Property,
    Constant,
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
            Self::Enum => write!(f, "enumeration"),
            Self::EnumStruct => write!(f, "enumstruct"),
            Self::Field => write!(f, "field"),
            Self::TypeSet => write!(f, "typeset"),
            Self::TypeDefinition => write!(f, "typedef"),
        }
    }
}
