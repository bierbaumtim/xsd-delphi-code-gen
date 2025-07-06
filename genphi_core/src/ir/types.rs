use std::collections::HashMap;

use crate::ir::{IrTypeIdOrName, type_registry::TypeRegistry};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DelphiType {
    List(Box<DelphiType>),
    Enum(IrTypeIdOrName),
    Class(IrTypeIdOrName),
    Binary,
    Boolean,
    DateTime,
    Double,
    Float,
    Integer,
    Pointer,
    String,
}

pub enum DelphiReturnType {
    Reference(String),
    Value(String),
    Array(String),
    Enum(String),
    BuiltIn(String),
    Void,
}

/// Represents visibility levels in Delphi
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DelphiVisibility {
    Private,
    StrictPrivate,
    Protected,
    Public,
    Published,
}

/// Represents the entire Delphi unit
#[derive(Debug, Default, Clone, PartialEq)]
pub struct DelphiUnit {
    pub unit_name: String,
    pub enums: Vec<DelphiEnum>,
    pub records: Vec<DelphiRecord>,
    pub classes: Vec<DelphiClass>,
    pub uses_interface: Vec<String>,
    pub uses_implementation: Vec<String>,
    pub comment: Option<String>,
}

/// Represents a Delphi class‘
#[derive(Debug, Clone, PartialEq)]
pub struct DelphiClass {
    pub name: String,
    pub internal_name: String, // Internal name for the class, used for code generation
    pub parent_class: Option<String>,
    pub fields: Vec<DelphiField>,
    pub methods: Vec<DelphiMethod>,
    pub properties: Vec<DelphiProperty>,
    pub generate_json_support: bool,
    pub generate_xml_support: bool,
    pub comment: Option<String>,
}

/// Represents a Delphi record
#[derive(Debug, Clone, PartialEq)]
pub struct DelphiRecord {
    pub name: String,
    pub fields: Vec<DelphiField>,
    pub methods: Vec<DelphiMethod>,
    pub generate_json_support: bool,
    pub generate_xml_support: bool,
    pub comment: Option<String>,
}

/// Represents a Delphi enumeration with helper
#[derive(Debug, Clone, PartialEq)]
pub struct DelphiEnum {
    pub name: String,
    pub internal_name: String, // Internal name for the enum, used for code generation
    pub variants: Vec<DelphiEnumVariant>,
    pub comment: Option<String>,
}

/// Represents a Delphi enumeration variant
#[derive(Debug, Clone, PartialEq)]
pub struct DelphiEnumVariant {
    pub name: String,
    pub value: Option<String>, // For explicit values like (Active = 1)
    pub comment: Option<String>,
}

/// Represents a Delphi method
#[derive(Debug, Clone, PartialEq)]
pub struct DelphiMethod {
    pub name: String,
    pub parameters: Vec<DelphiParameter>,
    pub return_type: Option<String>,
    pub visibility: DelphiVisibility,
    pub is_constructor: bool,
    pub is_class_method: bool,
    pub is_static: bool,
    pub is_virtual: bool,
    pub is_override: bool,
    pub comment: Option<String>,
}

/// Represents a method parameter
#[derive(Debug, Clone, PartialEq)]
pub struct DelphiParameter {
    pub name: String,
    pub param_type: DelphiType,
    pub is_const: bool,
    pub is_var: bool,
    pub is_out: bool,
    pub default_value: Option<String>,
}

/// Represents a Delphi property
#[derive(Debug, Clone, PartialEq)]
pub struct DelphiProperty {
    pub name: String,
    pub property_type: DelphiType,
    pub getter: Option<String>,
    pub setter: Option<String>,
    pub visibility: DelphiVisibility,
    pub comment: Option<String>,
}

/// Represents a field in a Delphi record or class
#[derive(Debug, Clone, PartialEq)]
pub struct DelphiField {
    pub name: String,
    pub field_type: DelphiType,
    pub visibility: DelphiVisibility,
    pub is_reference_type: bool,
    pub json_key: Option<String>,
    pub xml_attribute: Option<String>,
    pub comment: Option<String>,
    pub default_value: Option<String>,
    pub is_required: bool,
}

pub enum ExpressionOrStatement {
    Expression {},
}

impl DelphiClass {
    pub fn new<N, S>(name: N, source: S) -> Self
    where
        N: Into<String>,
        S: Into<String>,
    {
        let name = name.into();
        let source = source.into();
        let internal_name = TypeRegistry::build_internal_name(&name, &source);

        Self {
            name,
            internal_name,
            parent_class: None,
            fields: Vec::new(),
            methods: Vec::new(),
            properties: Vec::new(),
            generate_json_support: false,
            generate_xml_support: false,
            comment: None,
        }
    }
}

impl DelphiType {
    pub fn resolve(&self, registry: &TypeRegistry) -> Self {
        match self {
            DelphiType::Class(IrTypeIdOrName::Id(id)) => {
                registry.get_class_name_by_id(id).map_or_else(
                    || DelphiType::Pointer,
                    |n| DelphiType::Class(IrTypeIdOrName::Name(n.clone())),
                )
            }
            DelphiType::Enum(IrTypeIdOrName::Id(id)) => {
                registry.get_enum_name_by_id(id).map_or_else(
                    || DelphiType::Pointer,
                    |n| DelphiType::Enum(IrTypeIdOrName::Name(n.clone())),
                )
            }
            DelphiType::List(inner) => DelphiType::List(Box::new(inner.resolve(registry))),
            e => e.clone(),
        }
    }

    pub fn as_type_name(&self) -> String {
        match self {
            DelphiType::List(delphi_type) => {
                let list_type = if matches!(
                    delphi_type.as_ref(),
                    DelphiType::Class(_) | DelphiType::List(_)
                ) {
                    "TObjectList".to_owned()
                } else {
                    "TList".to_owned()
                };

                format!("{list_type}<{}>", delphi_type.as_type_name())
            }
            DelphiType::Enum(e) => match e {
                IrTypeIdOrName::Id(_) => {
                    unimplemented!("Support for type references in field types is not supported")
                }
                IrTypeIdOrName::Name(name) => name.clone(),
            },
            DelphiType::Class(c) => match c {
                IrTypeIdOrName::Id(_) => {
                    unimplemented!("Support for type references in field types is not supported")
                }
                IrTypeIdOrName::Name(name) => name.clone(),
            },
            DelphiType::String => "String".to_owned(),
            DelphiType::Integer => "Integer".to_owned(),
            DelphiType::Float => "Float".to_owned(),
            DelphiType::Double => "Double".to_owned(),
            DelphiType::Boolean => "Boolean".to_owned(),
            DelphiType::Pointer => "Pointer".to_owned(),
            DelphiType::DateTime => "TDateTime".to_owned(),
            DelphiType::Binary => "TBytes".to_owned(),
        }
    }
}
