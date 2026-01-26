use std::{
    fmt,
    io::{BufWriter, Write},
};

use super::internal_representation::InternalRepresentation;

/// Trait for code generators
pub trait CodeGenerator<T: Write> {
    fn new(
        buffer: BufWriter<T>,
        options: CodeGenOptions,
        internal_representation: InternalRepresentation,
        documentations: Vec<String>,
    ) -> Self;

    fn generate(&mut self) -> Result<(), CodeGenError>;
}

/// Options for the code generator
#[derive(Debug, Default, Clone)]
pub struct CodeGenOptions {
    /// Generate the `from_xml` function
    pub generate_from_xml: bool,

    /// Generate the `to_xml` function
    pub generate_to_xml: bool,

    /// The name of the unit
    pub unit_name: String,

    /// The prefix for the type
    pub type_prefix: Option<String>,

    /// Enable XSD validation code generation
    pub enable_validation: bool,

    /// Paths to XSD files for validation
    pub xsd_file_paths: Vec<std::path::PathBuf>,
}

/// Errors that can occur during code generation
pub enum CodeGenError {
    IOError(std::io::Error),
    ComplexTypeInSimpleTypeNotAllowed(String, String),
    TemplateEngineError(String),
    /// A required data type is missing
    MissingDataType(String, String),
    /// A fixed size list inside of a fixed size list is not supported
    NestedFixedSizeList(String, String),
    /// A list inside of a fixed size list is not supported
    NestedListInFixedSizeList(String, String),
}

impl From<std::io::Error> for CodeGenError {
    fn from(value: std::io::Error) -> Self {
        Self::IOError(value)
    }
}

impl fmt::Debug for CodeGenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IOError(arg0) => f.debug_tuple("IOError").field(arg0).finish(),
            Self::TemplateEngineError(e) => write!(f, "Some error occurred while using the template engine. Error: {}", e),
            Self::MissingDataType(type_name, variable)  => write!(
                f,
                "Required DataType is missing. Class: {type_name}, Variable: {variable}"
            ),
            Self::ComplexTypeInSimpleTypeNotAllowed(union_type, variant) => write!(
                f,
                "A complex type inside a union type is not supported. Class: {union_type}, Variable: {variant}"
            ),
            Self::NestedFixedSizeList(class, variable) => write!(
                f,
                "Fixed size list inside of a fixed size list is not supported. Class: {class}, Variable: {variable}"
            ),
            Self::NestedListInFixedSizeList(class, variable) => write!(
                f,
                "Lists inside of a fixed size list is not supported. Class: {class}, Variable: {variable}"
            ),
        }
    }
}
