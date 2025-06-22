use std::collections::HashMap;

/// Represents a Delphi enumeration variant
#[derive(Debug, Clone, PartialEq)]
pub struct DelphiEnumVariant {
    pub name: String,
    pub value: Option<String>, // For explicit values like (Active = 1)
    pub comment: Option<String>,
}

/// Represents a Delphi enumeration with helper
#[derive(Debug, Clone, PartialEq)]
pub struct DelphiEnum {
    pub name: String,
    pub variants: Vec<DelphiEnumVariant>,
    pub helper_name: String,
    pub generate_helper: bool,
    pub scoped: bool,
    pub comment: Option<String>,
}

/// Represents a field in a Delphi record or class
#[derive(Debug, Clone, PartialEq)]
pub struct DelphiField {
    pub name: String,
    pub field_type: String,
    pub visibility: DelphiVisibility,
    pub is_reference_type: bool,
    pub json_key: Option<String>,
    pub xml_attribute: Option<String>,
    pub comment: Option<String>,
    pub default_value: Option<String>,
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

/// Represents a method parameter
#[derive(Debug, Clone, PartialEq)]
pub struct DelphiParameter {
    pub name: String,
    pub param_type: String,
    pub is_const: bool,
    pub is_var: bool,
    pub is_out: bool,
    pub default_value: Option<String>,
}

/// Represents a Delphi method
#[derive(Debug, Clone, PartialEq)]
pub struct DelphiMethod {
    pub name: String,
    pub parameters: Vec<DelphiParameter>,
    pub return_type: Option<String>,
    pub visibility: DelphiVisibility,
    pub is_constructor: bool,
    pub is_destructor: bool,
    pub is_class_method: bool,
    pub is_static: bool,
    pub is_virtual: bool,
    pub is_override: bool,
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

/// Represents a Delphi class
#[derive(Debug, Clone, PartialEq)]
pub struct DelphiClass {
    pub name: String,
    pub parent_class: Option<String>,
    pub fields: Vec<DelphiField>,
    pub methods: Vec<DelphiMethod>,
    pub properties: Vec<DelphiProperty>,
    pub generate_json_support: bool,
    pub generate_xml_support: bool,
    pub comment: Option<String>,
}

/// Represents a Delphi property
#[derive(Debug, Clone, PartialEq)]
pub struct DelphiProperty {
    pub name: String,
    pub property_type: String,
    pub getter: Option<String>,
    pub setter: Option<String>,
    pub visibility: DelphiVisibility,
    pub comment: Option<String>,
}

/// Represents the entire Delphi unit
#[derive(Debug, Clone, PartialEq)]
pub struct DelphiUnit {
    pub unit_name: String,
    pub forward_declarations: Vec<String>,
    pub enums: Vec<DelphiEnum>,
    pub records: Vec<DelphiRecord>,
    pub classes: Vec<DelphiClass>,
    pub uses_interface: Vec<String>,
    pub uses_implementation: Vec<String>,
    pub constants: HashMap<String, String>,
    pub comment: Option<String>,
}
