#[derive(Clone, Debug)]
pub(crate) enum DataType {
    Boolean,
    DateTime,
    Date,
    Double,
    Binary(BinaryEncoding),
    /// i8: -127 to 128
    ShortInteger,
    /// i16: -32.768 to 32.767
    SmallInteger,
    /// i32: -2.147.483.648 to 2.147.483.647
    Integer,
    /// i64: -9.223.372.036.854.775.808 to 9.223.372.036.854.775.807
    LongInteger,
    /// u8: 0 to 255
    UnsignedShortInteger,
    /// u16: 0 to 65.535
    UnsignedSmallInteger,
    /// u32: 0 to 4.294.967.295
    UnsignedInteger,
    /// u64: 0 to 18.446.744.073.709.551.615
    UnsignedLongInteger,
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
