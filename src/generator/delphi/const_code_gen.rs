use std::io::{BufWriter, Write};

use crate::generator::{
    code_generator_trait::CodeGenError,
    types::{ClassType, DataType},
};

pub(crate) struct ConstCodeGenerator;

impl ConstCodeGenerator {
    pub(crate) fn generate<T: Write>(
        buffer: &mut BufWriter<T>,
        classes: &[ClassType],
    ) -> Result<(), CodeGenError> {
        let gen_bool_consts = classes.iter().any(|c| {
            c.variables
                .iter()
                .any(|v| matches!(v.data_type, DataType::Boolean))
        });

        if gen_bool_consts {
            buffer.write_all(b"const\n")?;
        }

        if gen_bool_consts {
            buffer.write_all(b"  cnXmlTrueValue: string = 'true';\n")?;
            buffer.write_all(b"  cnXmlFalseValue: string = 'false';\n")?;
        }

        Ok(())
    }
}
