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
    pub(crate) type_: Type,
    pub(crate) key: String,
    pub(crate) is_list_type: bool,
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

#[derive(Serialize, Eq, PartialEq)]
pub(crate) struct Endpoint {
    pub(crate) name: String,
    pub(crate) response_type: Type,
    pub(crate) args: Vec<EndpointArg>,
}

#[derive(Serialize, Eq, PartialEq)]
pub(crate) struct EndpointArg {
    pub(crate) name: String,
    pub(crate) type_name: String,
    pub(crate) arg_type: String,
}

#[derive(Serialize, Eq, PartialEq)]
pub(crate) struct Type {
    pub(crate) name: String,
    pub(crate) is_class: bool,
    pub(crate) is_enum: bool,
}

impl Default for Type {
    fn default() -> Self {
        Self {
            name: "none".to_string(),
            is_class: false,
            is_enum: false,
        }
    }
}
