use genphi_core::dependency_graph::Dependable;

#[derive(Clone, Debug)]
pub enum DataType {
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
pub enum BinaryEncoding {
    Hex,
    Base64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum XMLSource {
    Element,
    Attribute,
}

#[derive(Clone, Debug)]
pub struct Enumeration {
    pub name: String,
    pub qualified_name: String,
    pub values: Vec<EnumerationValue>,
    pub documentations: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct EnumerationValue {
    pub variant_name: String,
    pub xml_value: String,
    pub documentations: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct TypeAlias {
    pub name: String,
    pub qualified_name: String,
    pub for_type: DataType,
    pub pattern: Option<String>,
    pub documentations: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct ClassType {
    pub name: String,
    pub qualified_name: String,
    pub super_type: Option<(String, String)>,
    pub variables: Vec<Variable>,
    pub documentations: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct Variable {
    pub name: String,
    pub data_type: DataType,
    pub xml_name: String,
    pub requires_free: bool,
    pub required: bool,
    pub source: XMLSource,
    pub default_value: Option<String>,
    pub is_const: bool,
    pub documentations: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct UnionType {
    pub name: String,
    pub qualified_name: String,
    pub variants: Vec<UnionVariant>,
    pub documentations: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct UnionVariant {
    pub name: String,
    pub data_type: DataType,
}

impl Dependable<String> for ClassType {
    fn key(&self) -> &String {
        &self.name
    }

    fn key_and_deps(&self) -> (&String, Option<Vec<String>>) {
        (
            &self.name,
            self.super_type.as_ref().cloned().map(|(n, _)| vec![n]),
        )
    }
}

impl Dependable<String> for TypeAlias {
    fn key(&self) -> &String {
        &self.name
    }

    fn key_and_deps(&self) -> (&String, Option<Vec<String>>) {
        match &self.for_type {
            DataType::Custom(name) => (&self.name, Some(vec![name.clone()])),
            _ => (&self.name, None),
        }
    }
}

impl Dependable<String> for UnionType {
    fn key(&self) -> &String {
        &self.name
    }

    fn key_and_deps(&self) -> (&String, Option<Vec<String>>) {
        (
            &self.name,
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
