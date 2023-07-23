use std::fs::File;

use super::internal_representation::InternalRepresentation;

pub(crate) trait CodeGenerator<'a> {
    fn new(
        file: &'a mut File,
        unit_name: String,
        internal_representation: InternalRepresentation,
    ) -> Self;

    fn generate(&mut self) -> Result<(), std::io::Error>;
}
