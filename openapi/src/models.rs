use serde::Serialize;

#[derive(Serialize, Eq, PartialEq)]
pub(crate) struct ClassType {
    pub(crate) name: String,
    pub(crate) properties: Vec<Property>,
    pub(crate) needs_destructor: bool,
}

#[derive(Serialize, Eq, PartialEq)]
pub(crate) struct Property {
    pub(crate) name: String,
    pub(crate) type_name: String,
    pub(crate) key: String,
    pub(crate) is_reference_type: bool,
    pub(crate) is_list_type: bool,
    pub(crate) is_enum_type: bool,
}

#[derive(Serialize, Eq, PartialEq)]
pub(crate) struct EnumType {
    pub(crate) name: String,
    pub(crate) variants: Vec<EnumVariant>,
}

#[derive(Serialize, Eq, PartialEq)]
pub(crate) struct EnumVariant {
    pub(crate) name: String,
    pub(crate) key: String,
}
