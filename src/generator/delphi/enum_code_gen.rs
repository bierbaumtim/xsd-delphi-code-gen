use std::io::Write;

use crate::generator::{
    code_generator_trait::{CodeGenError, CodeGenOptions},
    types::Enumeration,
};

use super::{code_writer::CodeWriter, helper::Helper};

pub(crate) struct EnumCodeGenerator;

impl EnumCodeGenerator {
    pub(crate) fn write_declarations<T: Write>(
        writer: &mut CodeWriter<T>,
        enumerations: &Vec<Enumeration>,
        options: &CodeGenOptions,
        indentation: usize,
    ) -> Result<(), CodeGenError> {
        if enumerations.is_empty() {
            return Ok(());
        }

        writer.writeln("{$REGION 'Enumerations'}", Some(indentation))?;
        for enumeration in enumerations {
            Self::generate_declaration(writer, enumeration, options, indentation)?;
        }
        writer.writeln("{$ENDREGION}", Some(indentation))?;

        writer.newline()?;
        writer.writeln("{$REGION 'Enumerations Helper'}", Some(indentation))?;
        for (i, enumeration) in enumerations.iter().enumerate() {
            Self::generate_helper_declaration(writer, enumeration, options, indentation)?;

            if i < enumerations.len() - 1 {
                writer.newline()?;
            }
        }
        writer.writeln("{$ENDREGION}", Some(indentation))?;

        Ok(())
    }

    pub(crate) fn write_implementation<T: Write>(
        writer: &mut CodeWriter<T>,
        enumerations: &Vec<Enumeration>,
        options: &CodeGenOptions,
    ) -> Result<(), CodeGenError> {
        if enumerations.is_empty() {
            return Ok(());
        }

        writer.writeln("{$REGION 'Enumerations Helper'}", None)?;
        for (i, enumeration) in enumerations.iter().enumerate() {
            Self::generate_helper_implementation(writer, enumeration, options)?;

            if i < enumerations.len() - 1 {
                writer.newline()?;
            }
        }
        writer.writeln("{$ENDREGION}", None)?;

        Ok(())
    }

    fn generate_declaration<T: Write>(
        writer: &mut CodeWriter<T>,
        enumeration: &Enumeration,
        options: &CodeGenOptions,
        indentation: usize,
    ) -> Result<(), CodeGenError> {
        let prefix = Helper::get_enum_variant_prefix(&enumeration.name);

        writer.writeln_fmt(
            format_args!("// XML Qualified Name: {}", enumeration.qualified_name),
            Some(indentation),
        )?;
        writer.writeln_fmt(
            format_args!(
                "{} = ({});",
                Helper::as_type_name(&enumeration.name, &options.type_prefix),
                enumeration
                    .values
                    .iter()
                    .map(|v| prefix.clone() + v.variant_name.to_ascii_uppercase().as_str())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Some(indentation),
        )?;

        Ok(())
    }

    fn generate_helper_declaration<T: Write>(
        writer: &mut CodeWriter<T>,
        enumeration: &Enumeration,
        options: &CodeGenOptions,
        indentation: usize,
    ) -> Result<(), CodeGenError> {
        let formatted_enum_name = Helper::as_type_name(&enumeration.name, &options.type_prefix);

        writer.writeln_fmt(
            format_args!(
                "{}Helper = record helper for {}",
                formatted_enum_name, formatted_enum_name,
            ),
            Some(indentation),
        )?;

        if options.generate_from_xml {
            writer.writeln_fmt(
                format_args!(
                    "class function FromXmlValue(const pXmlValue: String): {}; static;",
                    formatted_enum_name,
                ),
                Some(indentation + 2),
            )?;
        }

        if options.generate_to_xml {
            writer.writeln("function ToXmlValue: String;", Some(indentation + 2))?;
        }

        writer.writeln("end;", Some(indentation))?;

        Ok(())
    }

    fn generate_helper_implementation<T: Write>(
        writer: &mut CodeWriter<T>,
        enumeration: &Enumeration,
        options: &CodeGenOptions,
    ) -> Result<(), CodeGenError> {
        let formatted_enum_name = Helper::as_type_name(&enumeration.name, &options.type_prefix);

        if options.generate_from_xml {
            Self::generate_helper_from_xml(writer, enumeration, &formatted_enum_name)?;
        }

        if options.generate_from_xml && options.generate_to_xml {
            writer.newline()?;
        }

        if options.generate_to_xml {
            Self::generate_helper_to_xml(writer, enumeration, formatted_enum_name)?;
        }

        Ok(())
    }

    fn generate_helper_from_xml<T: Write>(
        writer: &mut CodeWriter<T>,
        enumeration: &Enumeration,
        formatted_enum_name: &String,
    ) -> Result<(), CodeGenError> {
        writer.writeln_fmt(
            format_args!(
                "class function {}Helper.FromXmlValue(const pXmlValue: String): {};",
                formatted_enum_name, formatted_enum_name,
            ),
            None,
        )?;
        writer.writeln("begin", None)?;
        let prefix = Helper::get_enum_variant_prefix(&enumeration.name);

        for (i, value) in enumeration.values.iter().enumerate() {
            writer.writeln_fmt(
                format_args!("if pXmlValue = '{}' then begin", value.xml_value,),
                if i == 0 { Some(2) } else { None },
            )?;
            writer.writeln_fmt(
                format_args!(
                    "Result := {}.{}{};",
                    formatted_enum_name,
                    prefix,
                    value.variant_name.to_ascii_uppercase(),
                ),
                Some(4),
            )?;
            writer.write("end", Some(2))?;

            if i < enumeration.values.len() - 1 {
                writer.write(" else ", None)?;
            }
        }

        writer.writeln(" else begin", None)?;
        writer.writeln_fmt(
            format_args!(
                "raise Exception.Create('\"' + pXmlValue + '\" is a unknown value for {}');",
                formatted_enum_name,
            ),
            Some(4),
        )?;
        writer.writeln("end;", Some(2))?;
        writer.writeln("end;", None)?;
        Ok(())
    }

    fn generate_helper_to_xml<T: Write>(
        writer: &mut CodeWriter<T>,
        enumeration: &Enumeration,
        formatted_enum_name: String,
    ) -> Result<(), CodeGenError> {
        let max_variant_len = enumeration
            .values
            .iter()
            .map(|v| v.variant_name.len())
            .max()
            .unwrap_or(1);

        writer.writeln_fmt(
            format_args!("function {}Helper.ToXmlValue: String;", formatted_enum_name,),
            None,
        )?;
        writer.writeln("begin", None)?;
        writer.writeln("case Self of", Some(2))?;
        for value in &enumeration.values {
            writer.writeln_fmt(
                format_args!(
                    "{}.{}{}{}: Result := '{}';",
                    formatted_enum_name,
                    Helper::get_enum_variant_prefix(&enumeration.name),
                    value.variant_name.to_ascii_uppercase(),
                    " ".repeat(max_variant_len - value.variant_name.len() + 1),
                    value.xml_value,
                ),
                Some(4),
            )?;
        }
        writer.writeln("end;", Some(2))?;
        writer.writeln("end;", None)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // use super::*;

    // TODO: Write Test
}
