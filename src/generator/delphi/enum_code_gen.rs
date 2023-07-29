use std::{fs::File, io::Write};

use crate::generator::{code_generator_trait::CodeGenOptions, types::Enumeration};

use super::helper::Helper;

pub(crate) struct EnumCodeGenerator;

impl EnumCodeGenerator {
    pub(crate) fn write_declarations(
        file: &mut File,
        enumerations: &Vec<Enumeration>,
        options: &CodeGenOptions,
        indentation: usize,
    ) -> Result<(), std::io::Error> {
        file.write_all(b"  {$REGION 'Enumerations'}\n")?;
        for e in enumerations {
            Self::generate_declaration(file, e, indentation)?;
        }
        file.write_all(b"  {$ENDREGION}\n")?;

        file.write(b"\n")?;
        file.write_all(b"  {$REGION 'Enumerations Helper'}\n")?;
        for (i, e) in enumerations.iter().enumerate() {
            Self::generate_helper_declaration(file, e, options, indentation)?;

            if i < enumerations.len() - 1 {
                file.write_all(b"\n")?;
            }
        }
        file.write_all(b"  {$ENDREGION}\n")?;

        Ok(())
    }

    pub(crate) fn write_implementation(
        file: &mut File,
        enumerations: &Vec<Enumeration>,
        options: &CodeGenOptions,
    ) -> Result<(), std::io::Error> {
        file.write_all(b"{$REGION 'Enumerations Helper'}\n")?;
        for (i, enumeration) in enumerations.iter().enumerate() {
            Self::generate_helper_implementation(file, enumeration, options)?;

            if i < enumerations.len() - 1 {
                file.write_all(b"\n")?;
            }
        }
        file.write_all(b"{$ENDREGION}\n")?;

        Ok(())
    }

    fn generate_declaration(
        file: &mut File,
        enumeration: &Enumeration,
        indentation: usize,
    ) -> Result<(), std::io::Error> {
        file.write_fmt(format_args!(
            "{}T{} = ({});\n",
            " ".repeat(indentation),
            Helper::first_char_uppercase(&enumeration.name),
            enumeration
                .values
                .iter()
                .map(|v| Helper::first_char_lowercase(&v.variant_name))
                .collect::<Vec<String>>()
                .join(", ")
        ))
    }

    fn generate_helper_declaration(
        file: &mut File,
        enumeration: &Enumeration,
        options: &CodeGenOptions,
        indentation: usize,
    ) -> Result<(), std::io::Error> {
        let formatted_enum_name = Helper::first_char_uppercase(&enumeration.name);

        file.write_fmt(format_args!(
            "{}T{}Helper = record helper for T{}\n",
            " ".repeat(indentation),
            formatted_enum_name,
            formatted_enum_name,
        ))?;

        if options.generate_from_xml {
            file.write_fmt(format_args!(
                "{}class function FromXmlValue(const pXmlValue: String): T{}; static;\n",
                " ".repeat(indentation + 2),
                formatted_enum_name,
            ))?;
        }

        if options.generate_to_xml {
            file.write_fmt(format_args!(
                "{}function ToXmlValue: String;\n",
                " ".repeat(indentation + 2),
            ))?;
        }

        file.write_fmt(format_args!("{}end;\n", " ".repeat(indentation),))?;

        Ok(())
    }

    fn generate_helper_implementation(
        file: &mut File,
        enumeration: &Enumeration,
        options: &CodeGenOptions,
    ) -> Result<(), std::io::Error> {
        let formatted_enum_name = Helper::as_type_name(&enumeration.name);

        if options.generate_from_xml {
            Self::generate_helper_from_xml(file, enumeration, &formatted_enum_name)?;
        }

        if options.generate_from_xml && options.generate_to_xml {
            file.write_all(b"\n")?;
        }

        if options.generate_to_xml {
            Self::generate_helper_to_xml(file, enumeration, formatted_enum_name)?;
        }

        Ok(())
    }

    fn generate_helper_from_xml(
        file: &mut File,
        enumeration: &Enumeration,
        formatted_enum_name: &String,
    ) -> Result<(), std::io::Error> {
        let max_xml_value_len = enumeration
            .values
            .iter()
            .map(|v| v.xml_value.len() + 1)
            .max()
            .unwrap_or(4);

        file.write_fmt(format_args!(
            "class function {}Helper.FromXmlValue(const pXmlValue: String): {};\n",
            formatted_enum_name, formatted_enum_name,
        ))?;
        file.write_all(b"begin\n")?;
        file.write_all(b"  case pXmlValue of\n")?;
        for value in &enumeration.values {
            file.write_fmt(format_args!(
                "    '{}':{}Result := {}.{};\n",
                value.xml_value,
                " ".repeat(max_xml_value_len - value.xml_value.len()),
                formatted_enum_name,
                Helper::first_char_lowercase(&value.variant_name),
            ))?;
        }
        file.write_all(b"  end;\n")?;
        file.write_all(b"end;\n")?;
        Ok(())
    }

    fn generate_helper_to_xml(
        file: &mut File,
        enumeration: &Enumeration,
        formatted_enum_name: String,
    ) -> Result<(), std::io::Error> {
        let max_variant_len = enumeration
            .values
            .iter()
            .map(|v| v.variant_name.len() + 1)
            .max()
            .unwrap_or(4);

        file.write_fmt(format_args!(
            "function {}Helper.ToXmlValue: String;\n",
            formatted_enum_name,
        ))?;
        file.write_all(b"begin\n")?;
        file.write_all(b"  case Self of\n")?;
        for value in &enumeration.values {
            let formatted_variant_name = Helper::first_char_lowercase(&value.variant_name);

            file.write_fmt(format_args!(
                "    {}:{}Result := '{}';\n",
                formatted_variant_name,
                " ".repeat(max_variant_len - value.variant_name.len()),
                formatted_variant_name
            ))?;
        }
        file.write_all(b"    else Result := '';\n")?;
        file.write_all(b"  end;\n")?;
        file.write_all(b"end;\n")?;
        Ok(())
    }
}
