use std::io::Write;

use crate::generator::{
    code_generator_trait::CodeGenError,
    types::{ClassType, DataType},
};

use super::code_writer::CodeWriter;

pub(crate) struct ConstCodeGenerator;

impl ConstCodeGenerator {
    pub(crate) fn generate<T: Write>(
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
