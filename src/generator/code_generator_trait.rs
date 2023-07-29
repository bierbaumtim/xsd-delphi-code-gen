use std::fs::File;

use super::internal_representation::InternalRepresentation;

pub(crate) trait CodeGenerator<'a> {
    fn new(
        file: &'a mut File,
        options: CodeGenOptions,
        internal_representation: InternalRepresentation,
    ) -> Self;

    fn generate(&mut self) -> Result<(), std::io::Error>;
}

pub(crate) struct CodeGenOptions {
    pub(crate) generate_from_xml: bool,
    pub(crate) generate_to_xml: bool,
    pub(crate) unit_name: String,
}
