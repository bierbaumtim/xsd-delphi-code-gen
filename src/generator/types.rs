use super::dependency_graph::Dependable;

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
    Uri,
    Alias(String),
    Custom(String),
    Enumeration(String),
    List(Box<DataType>),
    FixedSizeList(Box<DataType>, usize),
    InlineList(Box<DataType>),
    // TODO: for later
    // InlineFixedSizeList(Box<DataType>, usize),
    Union(String),
}

#[derive(Clone, Debug)]
pub(crate) enum BinaryEncoding {
    Hex,
    Base64,
}

#[derive(Clone, Debug)]
pub(crate) struct Enumeration {
    pub(crate) name: String,
    pub(crate) qualified_name: String,
    pub(crate) values: Vec<EnumerationValue>,
    pub(crate) documentations: Vec<String>,
}

#[derive(Clone, Debug)]
pub(crate) struct EnumerationValue {
    pub(crate) variant_name: String,
    pub(crate) xml_value: String,
    pub(crate) documentations: Vec<String>,
}

#[derive(Clone, Debug)]
pub(crate) struct TypeAlias {
    pub(crate) name: String,
    pub(crate) qualified_name: String,
    pub(crate) for_type: DataType,
    pub(crate) pattern: Option<String>,
    pub(crate) documentations: Vec<String>,
}

#[derive(Clone, Debug)]
pub(crate) struct ClassType {
    pub(crate) name: String,
    pub(crate) qualified_name: String,
    pub(crate) super_type: Option<(String, String)>,
    pub(crate) variables: Vec<Variable>,
    pub(crate) documentations: Vec<String>,
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
    pub(crate) required: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct UnionType {
    pub(crate) name: String,
    pub(crate) qualified_name: String,
    pub(crate) variants: Vec<UnionVariant>,
    pub(crate) documentations: Vec<String>,
}

#[derive(Clone, Debug)]
pub(crate) struct UnionVariant {
    pub(crate) name: String,
    pub(crate) data_type: DataType,
}

impl Dependable<String> for ClassType {
    fn key(&self) -> &String {
        &self.qualified_name
    }

    fn key_and_deps(&self) -> (&String, Option<Vec<String>>) {
        (
            &self.qualified_name,
            self.super_type.as_ref().cloned().map(|(_, qn)| vec![qn]),
        )
    }
}

impl Dependable<String> for TypeAlias {
    fn key(&self) -> &String {
        &self.qualified_name
    }

    fn key_and_deps(&self) -> (&String, Option<Vec<String>>) {
        match &self.for_type {
            DataType::Custom(name) => (&self.qualified_name, Some(vec![name.clone()])),
            _ => (&self.qualified_name, None),
        }
    }
}

impl Dependable<String> for UnionType {
    fn key(&self) -> &String {
        &self.qualified_name
    }

    fn key_and_deps(&self) -> (&String, Option<Vec<String>>) {
        (
            &self.qualified_name,
            Some(
                self.variants
                    .iter()
                    .map(|v| match &v.data_type {
                        DataType::Union(n) => n.clone(),
                        _ => String::new(),
                    })
                    .filter(|d| !d.is_empty())
                    .collect::<Vec<String>>(),
            ),
        )
    }
}
