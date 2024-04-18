use std::{error::Error, fmt::Display};

/// xsd value for unbounded occurance is represented as -1
pub const UNBOUNDED_OCCURANCE: i64 = -1;
/// xsd default occurance is 1
pub const DEFAULT_OCCURANCE: i64 = 1;

#[derive(Debug)]
pub struct ParsedData {
    pub nodes: Vec<Node>,
    pub documentations: Vec<String>,
}

// xs:element
#[derive(Debug)]
pub struct Node {
    pub node_type: NodeType,
    pub name: String,
    pub base_attributes: BaseAttributes,
    /// Documentation extracted from xs:annotation
    pub documentations: Option<Vec<String>>,
}

impl Node {
    pub const fn new(
        node_type: NodeType,
        name: String,
        base_attributes: BaseAttributes,
        documentations: Option<Vec<String>>,
    ) -> Self {
        Self {
            node_type,
            name,
            base_attributes,
            documentations,
        }
    }
}

#[derive(Debug, Clone)]
pub enum NodeType {
    Standard(NodeBaseType),
    /// Contains qualified name of the type
    Custom(String),
}

/// base="" or type=""
#[derive(Debug, Clone)]
pub enum NodeBaseType {
    Boolean,
    DateTime,
    Date,
    Decimal,
    Double,
    Float,
    HexBinary,
    Base64Binary,
    /// i8: -127 to 128
    Byte,
    /// i16: -32.768 to 32.767
    Short,
    /// i32: -2.147.483.648 to 2.147.483.647
    Integer,
    /// i64: -9.223.372.036.854.775.808 to 9.223.372.036.854.775.807
    Long,
    /// u8: 0 to 255
    UnsignedByte,
    /// u16: 0 to 65.535
    UnsignedShort,
    /// u32: 0 to 4.294.967.295
    UnsignedInteger,
    /// u64: 0 to 18.446.744.073.709.551.615
    UnsignedLong,
    String,
    Time,
    Uri,
}

#[derive(Debug, Clone)]
pub struct BaseAttributes {
    pub min_occurs: Option<i64>,
    pub max_occurs: Option<i64>,
}

#[derive(Debug)]
pub enum CustomTypeDefinition {
    Simple(SimpleType),
    Complex(ComplexType),
}

impl CustomTypeDefinition {
    pub fn get_name(&self) -> String {
        match self {
            Self::Simple(t) => t.name.clone(),
            Self::Complex(t) => t.name.clone(),
        }
    }

    pub fn get_qualified_name(&self) -> String {
        match self {
            Self::Simple(t) => t.qualified_name.clone(),
            Self::Complex(t) => t.qualified_name.clone(),
        }
    }
}

impl From<SimpleType> for CustomTypeDefinition {
    fn from(value: SimpleType) -> Self {
        Self::Simple(value)
    }
}

impl From<ComplexType> for CustomTypeDefinition {
    fn from(value: ComplexType) -> Self {
        Self::Complex(value)
    }
}

/// types in xs:union
#[derive(Debug, Clone)]
pub enum UnionVariant {
    Standard(NodeBaseType),
    Named(String),
    Simple(SimpleType),
}

/// xs:simpleType
#[derive(Debug, Clone)]
pub struct SimpleType {
    /// name-attribute
    pub name: String,
    /// namespace + name
    pub qualified_name: String,

    /// Documentation extracted from xs:annotation
    pub documentations: Vec<String>,

    pub base_type: Option<NodeType>,
    /// possible values for an enumeration
    pub enumeration: Option<Vec<EnumerationVariant>>,
    /// type of items in a list
    pub list_type: Option<NodeType>,
    /// type of items in a list
    pub pattern: Option<String>,
    /// variants of union type
    pub variants: Option<Vec<UnionVariant>>,
}

/// xs:enumeration
#[derive(Debug, Clone)]
pub struct EnumerationVariant {
    /// Variant name
    pub name: String,
    /// Documentation
    pub documentations: Vec<String>,
}

/// xs:complexType
#[derive(Debug)]
pub struct ComplexType {
    /// name-attribute
    pub name: String,
    /// namespace + name
    pub qualified_name: String,

    /// Documentation extracted from xs:annotation
    pub documentations: Vec<String>,

    /// qualified name of another complex type
    pub base_type: Option<String>,
    /// elements of the complex type
    pub children: Vec<Node>,
    /// custom attributes of the complex type
    pub custom_attributes: Vec<CustomAttribute>,
    /// order of elements
    pub order: OrderIndicator,
}

#[derive(Debug)]
pub enum OrderIndicator {
    All,
    Choice(BaseAttributes),
    Sequence,
}

/// xs:attribute
#[derive(Debug)]
pub struct CustomAttribute {
    /// name-attribute
    pub name: String,
    /// namespace + name
    pub qualified_name: String,

    /// Documentation extracted from xs:annotation
    pub documentations: Vec<String>,

    pub base_type: NodeType,

    /// default value for the attribute
    pub default_value: Option<String>,

    /// const value for the attribute
    pub fixed_value: Option<String>,

    /// use-attribute (required or optional)
    pub required: bool,
}

#[derive(Debug, Clone)]
pub enum ParserError {
    FailedToResolveNamespace(String),
    MalformedAttribute(String, Option<String>),
    MalformedNamespaceAttribute(String),
    MissingOrNotSupportedBaseType(String),
    MissingAttribute(String),
    UnableToReadFile,
    UnexpectedEndOfFile,
    UnexpectedError,
    UnexpectedStartOfNode(String),
}

impl Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FailedToResolveNamespace(namespace) => {
                write!(f, "Namespace \"{namespace}\" could not be resolved")
            }
            Self::MalformedAttribute(name, reason) => write!(
                f,
                "Attribute \"{name}\" is malformed. Error: \"{reason:?}\""
            ),
            Self::MalformedNamespaceAttribute(message) => {
                write!(f, "Namespace attribute is malformed: \"{message}\"")
            }
            Self::MissingOrNotSupportedBaseType(value) => {
                write!(f, "Type is missing or unsupported \"{value}\"")
            }
            Self::MissingAttribute(name) => write!(f, "Missing Attribute \"{name}\""),
            Self::UnableToReadFile => write!(f, "Failed to read input file"),
            Self::UnexpectedEndOfFile => write!(f, "File ended to early"),
            Self::UnexpectedError => write!(f, "An unexpected error occured"),
            Self::UnexpectedStartOfNode(name) => write!(f, "Unexpected start of \"{name}\""),
        }
    }
}

impl Error for ParserError {}
