use serde::{Serialize, Deserialize};

pub mod base;
pub mod argument;
pub mod constant;
pub mod enumeration;
pub mod function;
pub mod method_map;
pub mod property;
pub mod type_def;
pub mod type_set;

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
    MethodMapMethod,
    MethodMapProperty,
    Constant,
    Enum,
    EnumStruct,
    EnumStructMethod,
    TypeSet,
    TypeDefinition,
}
