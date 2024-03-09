use std::io::Write;

use crate::generator::{
    code_generator_trait::CodeGenError,
    types::{ClassType, DataType},
};

use super::code_writer::CodeWriter;

/// A helper struct to generate the const section of the code.
pub struct ConstCodeGenerator;

impl ConstCodeGenerator {
    /// Generate the const section of the code.
    ///
    /// At the moment this only generates the boolean constants.
    pub fn generate<T: Write>(
        writer: &mut CodeWriter<T>,
        classes: &[ClassType],
    ) -> Result<(), CodeGenError> {
        let gen_bool_consts = classes.iter().any(|c| {
            c.variables
                .iter()
                .any(|v| matches!(v.data_type, DataType::Boolean))
        });

        if gen_bool_consts {
            writer.writeln("const", None)?;
        }

        if gen_bool_consts {
            writer.writeln("cnXmlTrueValue: string = 'true';", Some(2))?;
            writer.writeln("cnXmlFalseValue: string = 'false';", Some(2))?;
        }

        Ok(())
    }
}
