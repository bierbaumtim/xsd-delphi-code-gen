use std::io::{BufWriter, Write};

use crate::generator::{
    code_generator_trait::{CodeGenOptions, CodeGenerator},
    internal_representation::InternalRepresentation,
    types::*,
};

use super::{
    alias_code_gen::TypeAliasCodeGenerator, class_code_gen::ClassCodeGenerator,
    enum_code_gen::EnumCodeGenerator, helper_code_gen::HelperCodeGenerator,
};

// TODO: build IR(Intermediate Representation) with more informations about DataType, Inheritance
// TODO: Sort class Declarations by occurance in document, then by inheritance and dependency

pub(crate) struct DelphiCodeGenerator<'a> {
    buffer: &'a mut BufWriter<Box<dyn Write>>,
    options: CodeGenOptions,
    internal_representation: InternalRepresentation,
    generate_date_time_helper: bool,
    generate_hex_binary_helper: bool,
}

impl<'a> DelphiCodeGenerator<'a> {
    fn write_unit(&mut self) -> Result<(), std::io::Error> {
        self.buffer
            .write_fmt(format_args!("unit {};", self.options.unit_name))?;
        self.newline()?;
        self.newline()
    }

    fn write_uses(&mut self) -> Result<(), std::io::Error> {
        self.buffer.write_all(b"uses System.DateUtils,\n")?;
        self.buffer.write_all(b"     System.Types,\n")?;
        self.buffer.write_all(b"     System.Xml;")?;
        self.newline()?;
        self.newline()
    }

    fn write_interface_start(&mut self) -> Result<(), std::io::Error> {
        self.buffer.write_all(b"interface")?;
        self.newline()?;
        self.newline()
    }

    fn write_forward_declerations(&mut self) -> Result<(), std::io::Error> {
        self.buffer.write(b"types")?;
        self.newline()?;
        self.newline()?;

        if !self.internal_representation.enumerations.is_empty() {
            EnumCodeGenerator::write_declarations(
                self.buffer,
                &self.internal_representation.enumerations,
                &self.options,
                2,
            )?;
            self.newline()?;
        }

        if !self.internal_representation.types_aliases.is_empty() {
            TypeAliasCodeGenerator::write_declarations(
                self.buffer,
                &self.internal_representation.types_aliases,
                2,
            )?;
            self.newline()?;
        }

        if !self.internal_representation.classes.is_empty() {
            ClassCodeGenerator::write_forward_declerations(
                self.buffer,
                &self.internal_representation.classes,
                2,
            )?;
            self.newline()?;
        }

        Ok(())
    }

    fn write_declarations(&mut self) -> Result<(), std::io::Error> {
        ClassCodeGenerator::write_declarations(
            self.buffer,
            &self.internal_representation.classes,
            &self.internal_representation.document,
            &self.options,
            2,
        )?;

        Ok(())
    }

    fn write_implementation_start(&mut self) -> Result<(), std::io::Error> {
        self.buffer.write_all(b"implementation")?;
        self.newline()?;
        self.newline()
    }

    fn write_implementation(&mut self) -> Result<(), std::io::Error> {
        EnumCodeGenerator::write_implementation(
            self.buffer,
            &self.internal_representation.enumerations,
            &self.options,
        )?;
        self.newline()?;

        HelperCodeGenerator::write(
            self.buffer,
            &self.options,
            self.generate_date_time_helper,
            self.generate_hex_binary_helper,
        )?;

        ClassCodeGenerator::write_implementations(
            self.buffer,
            &self.internal_representation.classes,
            &self.internal_representation.document,
            &self.internal_representation.types_aliases,
            &self.options,
        )?;

        self.newline()?;

        Ok(())
    }

    fn write_file_end(&mut self) -> Result<(), std::io::Error> {
        self.buffer.write_all(b"end.")
    }

    fn newline(&mut self) -> Result<(), std::io::Error> {
        self.buffer.write_all(b"\n")
    }
}

impl<'a> CodeGenerator<'a> for DelphiCodeGenerator<'a> {
    fn new(
        buffer: &'a mut BufWriter<Box<dyn Write>>,
        options: CodeGenOptions,
        internal_representation: InternalRepresentation,
    ) -> Self {
        DelphiCodeGenerator {
            buffer,
            options,
            generate_date_time_helper: internal_representation.classes.iter().any(|c| {
                c.variables.iter().any(|v| match &v.data_type {
                    DataType::DateTime | DataType::Date | DataType::Time => true,
                    _ => false,
                })
            }) || internal_representation.types_aliases.iter().any(
                |a| match &a.for_type {
                    DataType::DateTime | DataType::Date | DataType::Time => true,
                    _ => false,
                },
            ),
            generate_hex_binary_helper: internal_representation.classes.iter().any(|c| {
                c.variables.iter().any(|v| match &v.data_type {
                    DataType::Binary(BinaryEncoding::Hex) => true,
                    _ => false,
                })
            }) || internal_representation.types_aliases.iter().any(
                |a| match &a.for_type {
                    DataType::Binary(BinaryEncoding::Hex) => true,
                    _ => false,
                },
            ),
            internal_representation,
        }
    }

    fn generate(&mut self) -> Result<(), std::io::Error> {
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
