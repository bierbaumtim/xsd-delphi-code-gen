use serde::Serialize;

#[derive(Clone, Debug, Serialize, Eq, PartialEq)]
pub struct ClassType<'a> {
    pub name: String,
    pub qualified_name: &'a String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub super_type: Option<String>,
    pub documentations: Vec<&'a str>,
    // variables
    pub variables: Vec<Variable<'a>>,
    pub optional_variables: Vec<Variable<'a>>,
    pub constant_variables: Vec<Variable<'a>>,
    pub serialize_variables: Vec<SerializeVariable<'a>>,
    // initializer
    pub variable_initializer: Vec<String>,
    // deserialize
    pub has_optional_element_variables: bool,
    pub deserialize_attribute_variables: Vec<AttributeDeserializeVariable<'a>>,
    pub deserialize_element_variables: Vec<ElementDeserializeVariable<'a>>,
    //
    pub needs_destructor: bool,
    pub has_optional_fields: bool,
    pub has_constant_fields: bool,
}

#[derive(Clone, Debug, Serialize, Eq, PartialEq)]
pub struct Variable<'a> {
    pub name: String,
    pub data_type_repr: String,
    pub xml_name: &'a String,
    pub requires_free: bool,
    pub required: bool,
    pub default_value: &'a Option<String>,
    pub documentations: Vec<&'a str>,
}

#[derive(Clone, Debug, Serialize, Eq, PartialEq)]
pub struct SerializeVariable<'a> {
    pub name: String,
    pub xml_name: &'a String,
    //
    pub is_class: bool,
    pub is_enum: bool,
    pub is_list: bool,
    pub is_inline_list: bool,
    pub is_required: bool,
    pub has_optional_wrapper: bool,
    pub from_xml_code: String,
    pub to_xml_code: String,
}

#[derive(Clone, Debug, Serialize, Eq, PartialEq)]
pub struct AttributeDeserializeVariable<'a> {
    pub name: String,
    pub xml_name: &'a String,
    //
    pub has_optional_wrapper: bool,
    pub from_xml_code_available: String,
    pub from_xml_code_missing: String,
}

#[derive(Clone, Debug, Serialize, Eq, PartialEq)]
pub struct ElementDeserializeVariable<'a> {
    pub name: String,
    pub xml_name: &'a String,
    //
    pub is_required: bool,
    pub is_list: bool,
    pub is_inline_list: bool,
    pub is_fixed_size_list: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fixed_size_list_size: Option<usize>,
    pub has_optional_wrapper: bool,
    pub data_type_repr: String,
    pub from_xml_code: String,
}

#[derive(Clone, Debug, Serialize, Eq, PartialEq)]
pub struct Enumeration<'a> {
    pub name: String,
    pub qualified_name: &'a String,
    pub values: Vec<EnumerationValue<'a>>,
    pub documentations: Vec<&'a str>,
    //
    pub variant_prefix: String,
    pub line_per_variant: bool,
}

#[derive(Clone, Debug, Serialize, Eq, PartialEq)]
pub struct EnumerationValue<'a> {
    pub variant_name: String,
    pub xml_value: &'a String,
    pub documentations: Vec<&'a str>,
}

#[derive(Clone, Debug, Serialize, Eq, PartialEq)]
pub struct TypeAlias<'a> {
    pub name: String,
    pub qualified_name: &'a String,
    pub data_type_repr: String,
    pub pattern: &'a Option<String>,
    pub documentations: Vec<&'a str>,
}

#[derive(Clone, Debug, Serialize, Eq, PartialEq)]
pub struct UnionType<'a> {
    pub name: String,
    pub qualified_name: &'a String,
    pub variants: Vec<UnionVariant>,
    pub documentations: Vec<&'a str>,
}

#[derive(Clone, Debug, Serialize, Eq, PartialEq)]
pub struct UnionVariant {
    pub name: String,
    pub variable_name: String,
    pub data_type_repr: String,
    //
    pub is_list_type: bool,
    pub is_inline_list: bool,
    pub use_to_xml_func: bool,
    pub value_as_str_repr: String,
}
