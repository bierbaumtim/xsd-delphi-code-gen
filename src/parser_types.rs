use std::{error::Error, fmt::Display};

pub(crate) const UNBOUNDED_OCCURANCE: i64 = -1;
pub(crate) const DEFAULT_OCCURANCE: i64 = 1;

// xs:element
#[derive(Debug)]
pub(crate) struct Node {
    pub(crate) node_type: NodeType,
    pub(crate) name: String,
    pub(crate) base_attributes: BaseAttributes,
}

impl Node {
    pub(crate) fn new(node_type: NodeType, name: String, base_attributes: BaseAttributes) -> Node {
        Node {
            node_type,
            name,
            base_attributes,
        }
    }
}

#[derive(Debug)]
pub(crate) enum NodeType {
    Standard(NodeBaseType),
    /// Contains qualified name of the type
    Custom(String),
}

/// base="" or type=""
#[derive(Debug)]
pub(crate) enum NodeBaseType {
    Boolean,
    DateTime,
    Date,
    Decimal,
    Double,
    Float,
    HexBinary,
    Base64Binary,
    Integer,
    String,
    Time,
}

#[derive(Debug)]
pub(crate) struct BaseAttributes {
    pub(crate) min_occurs: Option<i64>,
    pub(crate) max_occurs: Option<i64>,
}

#[derive(Debug)]
pub(crate) enum CustomTypeDefinition {
    Simple(SimpleType),
    Complex(ComplexType),
}

impl CustomTypeDefinition {
    pub(crate) fn get_name(&self) -> String {
        match self {
            CustomTypeDefinition::Simple(t) => t.name.clone(),
            CustomTypeDefinition::Complex(t) => t.name.clone(),
        }
    }

    pub(crate) fn get_qualified_name(&self) -> String {
        match self {
            CustomTypeDefinition::Simple(t) => match &t.qualified_name {
                Some(v) => v.clone(),
                None => t.name.clone(),
            },
            CustomTypeDefinition::Complex(t) => match &t.qualified_name {
                Some(v) => v.clone(),
                None => t.name.clone(),
            },
        }
    }
}

/// xs:simpleType
#[derive(Debug)]
pub(crate) struct SimpleType {
    /// name-attribute
    pub(crate) name: String,
    /// namespace + name
    pub(crate) qualified_name: Option<String>,

    pub(crate) base_type: Option<NodeType>,
    /// possible values for an enumeration
    pub(crate) enumeration: Option<Vec<String>>,
    /// type of items in a list
    pub(crate) list_type: Option<NodeType>,
    /// type of items in a list
    pub(crate) pattern: Option<String>,
}

/// xs:complexType
#[derive(Debug)]
pub(crate) struct ComplexType {
    /// name-attribute
    pub(crate) name: String,
    /// namespace + name
    pub(crate) qualified_name: Option<String>,
    /// qualified name of another complex type
    pub(crate) base_type: Option<String>,
    /// elements of the complex type
    pub(crate) children: Vec<Node>,
}

#[derive(Debug, Clone)]
pub(crate) enum ParserError {
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
                write!(f, "Namespace \"{}\" could not be resolved", namespace)
            }
            Self::MalformedAttribute(name, reason) => write!(
                f,
                "Attribute \"{}\" is malformed. Error: \"{:?}\"",
                name, reason
            ),
            Self::MalformedNamespaceAttribute(message) => {
                write!(f, "Namespace attribute is malformed: \"{}\"", message)
            }
            Self::MissingOrNotSupportedBaseType(value) => {
                write!(f, "Type is missing or unsupported \"{}\"", value)
            }
            Self::MissingAttribute(name) => write!(f, "Missing Attribute \"{}\"", name),
            Self::UnableToReadFile => write!(f, "Failed to read input file"),
            Self::UnexpectedEndOfFile => write!(f, "File ended to early"),
            Self::UnexpectedError => write!(f, "An unexpected error occured"),
            Self::UnexpectedStartOfNode(name) => write!(f, "Unexpected start of \"{}\"", name),
        }
    }
}

impl Error for ParserError {}
