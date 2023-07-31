#[derive(Clone, Debug)]
pub(crate) enum DataType {
    Boolean,
    DateTime,
    Date,
    Double,
    Binary(BinaryEncoding),
    Integer,
    String,
    Time,
    Alias(String),
    Enumeration(String),
    Custom(String),
    List(Box<DataType>),
    FixedSizeList(Box<DataType>, usize),
}

#[derive(Clone, Debug)]
pub(crate) enum BinaryEncoding {
    Hex,
    Base64,
}

pub(crate) struct Enumeration {
    pub(crate) name: String,
    pub(crate) values: Vec<EnumerationValue>,
}

pub(crate) struct EnumerationValue {
    pub(crate) variant_name: String,
    pub(crate) xml_value: String,
}

#[derive(Clone, Debug)]
pub(crate) struct TypeAlias {
    pub(crate) name: String,
    pub(crate) for_type: DataType,
    pub(crate) pattern: Option<String>,
}

#[derive(Clone, Debug)]
pub(crate) struct ClassType {
    pub(crate) name: String,
    pub(crate) super_type: Option<String>,
    pub(crate) variables: Vec<Variable>,
    // local_types: Vec<ClassType>,
    // type_aliases: Vec<TypeAlias>,
    // enumerations: Vec<Enumeration>,
}

#[derive(Clone, Debug)]
pub(crate) struct Variable {
    pub(crate) name: String,
    pub(crate) data_type: DataType,
    pub(crate) xml_name: String,
    pub(crate) requires_free: bool,
}
