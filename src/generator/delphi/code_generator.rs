use chrono::Local;
use std::io::{BufWriter, Write};

use crate::generator::{
    code_generator_trait::{CodeGenError, CodeGenOptions, CodeGenerator},
    internal_representation::InternalRepresentation,
    types::*,
};

use super::{
    alias_code_gen::TypeAliasCodeGenerator, class_code_gen::ClassCodeGenerator,
    code_writer::CodeWriter, const_code_gen::ConstCodeGenerator, enum_code_gen::EnumCodeGenerator,
    helper_code_gen::HelperCodeGenerator, union_type_code_gen::UnionTypeCodeGenerator,
};

pub(crate) struct DelphiCodeGenerator<T: Write> {
    writer: CodeWriter<T>,
    options: CodeGenOptions,
    internal_representation: InternalRepresentation,
    documentations: Vec<String>,
    generate_date_time_helper: bool,
    generate_hex_binary_helper: bool,
}

impl<T: Write> DelphiCodeGenerator<T> {
    #[inline]
    fn write_header_comment(&mut self) -> Result<(), std::io::Error> {
        const MAX_LENGTH: usize = 80;
        const DELIMITER_LENGTH: usize = 3;
        const BORDER_LENGTH: usize = MAX_LENGTH - (2 * DELIMITER_LENGTH);
        const GENERATED_CONTENT: &str = "Generated by XSD Delphi Code Gen";

        let version_content = format!(
            "Version: {}",
            option_env!("CARGO_PKG_VERSION").unwrap_or("unknown")
        );
        let timestamp = format!("Timestamp: {}", Local::now().format("%d.%m.%Y %H:%M:%S"));

        // TODO: GenerateCommentBlock(line_length: usize, lines: Vec<String>)
        self.writer
            .writeln_fmt(format_args!("// {} //", "=".repeat(BORDER_LENGTH)), None)?;
        self.writer.writeln_fmt(
            format_args!(
                "// {}{} //",
                GENERATED_CONTENT,
                " ".repeat(BORDER_LENGTH - GENERATED_CONTENT.len())
            ),
            None,
        )?;
        self.writer.writeln_fmt(
            format_args!(
                "// {}{} //",
                version_content,
                " ".repeat(BORDER_LENGTH - version_content.len())
            ),
            None,
        )?;
        self.writer.writeln_fmt(
            format_args!(
                "// {}{} //",
                timestamp,
                " ".repeat(BORDER_LENGTH - timestamp.len())
            ),
            None,
        )?;
        self.writer
            .writeln_fmt(format_args!("// {} //", "=".repeat(BORDER_LENGTH)), None)?;
        self.writer.newline()?;

        Ok(())
    }

    #[inline]
    fn write_documentations(&mut self) -> Result<(), std::io::Error> {
        if !self.documentations.is_empty() {
            self.writer.newline()?;
            self.writer
                .write_documentation(&self.documentations, None)?;
            self.writer.newline()?;
        }

        Ok(())
    }

    #[inline]
    fn write_unit(&mut self) -> Result<(), std::io::Error> {
        self.writer
            .writeln(format!("unit {};", self.options.unit_name).as_str(), None)?;
        self.writer.newline()
    }

    #[inline]
    fn write_uses(&mut self) -> Result<(), std::io::Error> {
        self.writer.writeln("uses System.DateUtils,", None)?;
        self.writer
            .writeln("System.Generics.Collections,", Some(5))?;
        self.writer.writeln("System.Types,", Some(5))?;
        self.writer.writeln("System.StrUtils,", Some(5))?;
        self.writer.writeln("System.SysUtils,", Some(5))?;
        self.writer.writeln("Xml.XMLDoc,", Some(5))?;
        self.writer.writeln("Xml.XMLIntf;", Some(5))?;
        self.writer.newline()
    }

    #[inline]
    fn write_interface_start(&mut self) -> Result<(), std::io::Error> {
        self.writer.writeln("interface", None)?;
        self.writer.newline()
    }

    #[inline]
    fn write_forward_declerations(&mut self) -> Result<(), CodeGenError> {
        self.writer.writeln("type", None)?;

        if !self.internal_representation.enumerations.is_empty() {
            EnumCodeGenerator::write_declarations(
                &mut self.writer,
                &self.internal_representation.enumerations,
                &self.options,
                2,
            )?;
            self.writer.newline()?;
        }

        if !self.internal_representation.classes.is_empty() {
            ClassCodeGenerator::write_forward_declerations(
                &mut self.writer,
                &self.internal_representation.classes,
                &self.options,
                2,
            )?;
            self.writer.newline()?;
        }

        if !self.internal_representation.types_aliases.is_empty() {
            TypeAliasCodeGenerator::write_declarations(
                &mut self.writer,
                &self.internal_representation.types_aliases,
                &self.options,
                2,
            )?;
            self.writer.newline()?;
        }

        if !self.internal_representation.union_types.is_empty() {
            UnionTypeCodeGenerator::write_declarations(
                &mut self.writer,
                &self.internal_representation.union_types,
                &self.options,
                2,
            )?;
            self.writer.newline()?;
        }

        Ok(())
    }

    #[inline]
    fn write_declarations(&mut self) -> Result<(), CodeGenError> {
        ClassCodeGenerator::write_declarations(
            &mut self.writer,
            &self.internal_representation.classes,
            &self.internal_representation.document,
            &self.options,
            2,
        )?;

        Ok(())
    }

    #[inline]
    fn write_implementation_start(&mut self) -> Result<(), std::io::Error> {
        self.writer.write("implementation", None)?;
        self.writer.newline()?;
        self.writer.newline()
    }

    #[inline]
    fn write_implementation(&mut self) -> Result<(), CodeGenError> {
        ConstCodeGenerator::generate(&mut self.writer, &self.internal_representation.classes)?;
        self.writer.newline()?;

        EnumCodeGenerator::write_implementation(
            &mut self.writer,
            &self.internal_representation.enumerations,
            &self.options,
        )?;
        self.writer.newline()?;

        HelperCodeGenerator::write(
            &mut self.writer,
            &self.options,
            self.generate_date_time_helper,
            self.generate_hex_binary_helper,
        )?;

        UnionTypeCodeGenerator::write_implementations(
            &mut self.writer,
            &self.internal_representation.union_types,
            &self.internal_representation.enumerations,
            &self.internal_representation.types_aliases,
            &self.options,
        )?;

        ClassCodeGenerator::write_implementations(
            &mut self.writer,
            &self.internal_representation.classes,
            &self.internal_representation.document,
            &self.internal_representation.types_aliases,
            &self.options,
        )?;

        self.writer.newline()?;

        Ok(())
    }

    #[inline]
    fn write_file_end(&mut self) -> Result<(), std::io::Error> {
        self.writer.write("end.", None)
    }
}

impl<T> CodeGenerator<T> for DelphiCodeGenerator<T>
where
    T: Write,
{
    fn new(
        buffer: BufWriter<T>,
        options: CodeGenOptions,
        internal_representation: InternalRepresentation,
        documentations: Vec<String>,
    ) -> Self {
        DelphiCodeGenerator {
            writer: CodeWriter { buffer },
            options,
            documentations,
            generate_date_time_helper: internal_representation.classes.iter().any(|c| {
                c.variables.iter().any(|v| {
                    matches!(
                        &v.data_type,
                        DataType::DateTime | DataType::Date | DataType::Time
                    )
                })
            }) || internal_representation.types_aliases.iter().any(
                |a| {
                    matches!(
                        &a.for_type,
                        DataType::DateTime | DataType::Date | DataType::Time
                    )
                },
            ),
            generate_hex_binary_helper: internal_representation.classes.iter().any(|c| {
                c.variables
                    .iter()
                    .any(|v| matches!(&v.data_type, DataType::Binary(BinaryEncoding::Hex)))
            }) || internal_representation
                .types_aliases
                .iter()
                .any(|a| matches!(&a.for_type, DataType::Binary(BinaryEncoding::Hex))),
            internal_representation,
        }
    }

    fn generate(&mut self) -> Result<(), CodeGenError> {
        self.write_header_comment()?;
        self.write_documentations()?;
        self.write_unit()?;
        self.write_interface_start()?;
        self.write_uses()?;

        self.write_forward_declerations()?;
        self.write_declarations()?;

        self.write_implementation_start()?;
        self.write_implementation()?;

        self.write_file_end()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // use super::*;

    // TODO: Write Test
}
