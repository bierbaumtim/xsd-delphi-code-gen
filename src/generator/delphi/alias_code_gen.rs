use std::io::{BufWriter, Write};

use crate::generator::types::TypeAlias;

use super::helper::Helper;

pub(crate) struct TypeAliasCodeGenerator;

impl TypeAliasCodeGenerator {
    pub(crate) fn write_declarations(
        buffer: &mut BufWriter<Box<dyn Write>>,
        type_aliases: &Vec<TypeAlias>,
        indentation: usize,
    ) -> Result<(), std::io::Error> {
        buffer.write_all(b"  {$REGION 'Aliases'}\n")?;
        for type_alias in type_aliases {
            buffer.write_fmt(format_args!(
                "{}T{} = {};\n",
                " ".repeat(indentation),
                type_alias.name,
                Helper::get_datatype_language_representation(&type_alias.for_type),
            ))?;
        }
        buffer.write_all(b"  {$ENDREGION}\n")?;

        Ok(())
    }
}
