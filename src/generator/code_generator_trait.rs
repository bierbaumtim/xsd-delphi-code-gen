use std::{
    fmt,
    io::{BufWriter, Write},
};

use super::internal_representation::InternalRepresentation;

pub(crate) trait CodeGenerator<'a, T: Write> {
    fn new(
        buffer: &'a mut BufWriter<T>,
        options: CodeGenOptions,
        internal_representation: InternalRepresentation,
    ) -> Self;

    fn generate(&mut self) -> Result<(), CodeGenError>;
}

#[derive(Debug, Default)]
pub(crate) struct CodeGenOptions {
    pub(crate) generate_from_xml: bool,
    pub(crate) generate_to_xml: bool,
    pub(crate) unit_name: String,
    pub(crate) type_prefix: Option<String>,
}

pub(crate) enum CodeGenError {
    IOError(std::io::Error),
    NestedFixedSizeList(String, String),
    NestedListInFixedSizeList(String, String),
}

impl From<std::io::Error> for CodeGenError {
    fn from(value: std::io::Error) -> Self {
        CodeGenError::IOError(value)
    }
}

impl fmt::Debug for CodeGenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IOError(arg0) => f.debug_tuple("IOError").field(arg0).finish(),
            Self::NestedFixedSizeList(class, variable) => write!(
                f,
                "Fixed size list inside of a fixed size list is not supported. Class: {}, Variable: {}",
                class, variable
            ),
            Self::NestedListInFixedSizeList(class, variable) => write!(
                f,
                "Lists inside of a fixed size list is not supported. Class: {}, Variable: {}",
                class, variable
            ),
        }
    }
}
