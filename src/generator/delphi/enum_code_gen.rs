use std::io::Write;

use crate::generator::{
    code_generator_trait::{CodeGenError, CodeGenOptions},
    types::Enumeration,
};

use super::{code_writer::CodeWriter, helper::Helper};

/// Generates the code for enumerations.
pub struct EnumCodeGenerator;

impl EnumCodeGenerator {
    /// Generates declarations for enumerations and their helper types.
    /// 
    /// # Arguments
    /// 
    /// * `writer` - The writer to write the declarations to.
    /// * `enumerations` - The enumerations to generate.
    /// * `options` - The code generation options.
    /// * `indentation` - The indentation level.
    /// 
    /// # Example
    /// 
    /// ```
    /// use std::io::BufWriter;
    /// 
    /// use xsd_parser::generator::{
    ///    code_generator_trait::CodeGenOptions,
    ///   types::{Enumeration, EnumerationValue},
    /// };
    /// 
    /// use xsd_parser::generator::delphi::code_writer::CodeWriter;
    /// use xsd_parser::generator::delphi::enum_code_gen::EnumCodeGenerator;
    /// 
    /// let enumerations = vec![
    ///   Enumeration {
    ///     name: "ItemStatus".to_owned(),
    ///     qualified_name: "ItemStatus".to_owned(),
    ///     documentations: vec![],
    ///     values: vec![
    ///       EnumerationValue {
    ///         variant_name: "open".to_owned(),
    ///         xml_value: "o".to_owned(),
    ///         documentations: vec![],
    ///       },
    ///       EnumerationValue {
    ///         variant_name: "closed".to_owned(),
    ///         xml_value: "c".to_owned(),
    ///         documentations: vec![],
    ///       },
    ///       EnumerationValue {
    ///         variant_name: "unknown".to_owned(),
    ///         xml_value: "u".to_owned(),
    ///         documentations: vec![],
    ///       },
    ///     ],
    ///   },
    ///   Enumeration {
    ///     name: "Fontsize".to_owned(),
    ///     qualified_name: "Fontsize".to_owned(),
    ///     documentations: vec![],
    ///     values: vec![
    ///       EnumerationValue {
    ///         variant_name: "small".to_owned(),
    ///         xml_value: "s".to_owned(),
    ///         documentations: vec![],
    ///       },
    ///       EnumerationValue {
    ///         variant_name: "medium".to_owned(),
    ///         xml_value: "m".to_owned(),
    ///         documentations: vec![],
    ///       },
    ///       EnumerationValue {
    ///         variant_name: "large".to_owned(),
    ///         xml_value: "l".to_owned(),
    ///         documentations: vec![],
    ///       },
    ///     ],
    ///   },
    /// ];
    /// 
    /// let options = CodeGenOptions {
    /// generate_from_xml: true,
    /// generate_to_xml: true,
    /// ..Default::default()
    /// };
    /// 
    /// let buffer = BufWriter::new(Vec::new());
    /// let mut writer = CodeWriter { buffer };
    /// EnumCodeGenerator::write_declarations(&mut writer, &enumerations, &options, 2).unwrap();
    /// 
    /// let bytes = writer.get_writer().unwrap().clone();
    /// let content = String::from_utf8(bytes).unwrap();
    /// 
    /// let expected = "  {$REGION 'Enumerations'}\n  \
    ///             // XML Qualified Name: ItemStatus\n  \
    ///             TItemStatus = (isOpen, isClosed, isUnknown);\n\
    ///             \n  \
    ///             // XML Qualified Name: Fontsize\n  \
    ///             TFontsize = (fSmall, fMedium, fLarge);\n  \
    ///             {$ENDREGION}\n\
    ///             \n  \
    ///             {$REGION 'Enumerations Helper'}\n  \
    ///             TItemStatusHelper = record helper for TItemStatus\n    \
    ///             class function FromXmlValue(const pXmlValue: String): TItemStatus; static;\n    \
    ///             function ToXmlValue: String;\n  \
    ///             end;\n\
    ///             \n  \
    ///             TFontsizeHelper = record helper for TFontsize\n    \
    ///             class function FromXmlValue(const pXmlValue: String): TFontsize; static;\n    \
    ///             function ToXmlValue: String;\n  \
    ///             end;\n  \
    ///             {$ENDREGION}\n";
    /// 
    /// assert_eq!(content, expected);
    /// ```
    pub fn write_declarations<T: Write>(
        writer: &mut CodeWriter<T>,
        enumerations: &Vec<Enumeration>,
        options: &CodeGenOptions,
        indentation: usize,
    ) -> Result<(), CodeGenError> {
        if enumerations.is_empty() {
            return Ok(());
        }

        writer.writeln("{$REGION 'Enumerations'}", Some(indentation))?;
        for (i, enumeration) in enumerations.iter().enumerate() {
            Self::generate_declaration(writer, enumeration, options, indentation)?;

            if i < enumerations.len() - 1 {
                writer.newline()?;
            }
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

    pub fn write_implementation<T: Write>(
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

        writer.write_documentation(&enumeration.documentations, Some(indentation))?;
        Helper::write_qualified_name_comment(
            writer,
            &enumeration.qualified_name,
            Some(indentation),
        )?;

        if enumeration
            .values
            .iter()
            .any(|v| !v.documentations.is_empty())
        {
            writer.writeln_fmt(
                format_args!(
                    "{} = (",
                    Helper::as_type_name(&enumeration.name, &options.type_prefix),
                ),
                Some(indentation),
            )?;

            for (i, value) in enumeration.values.iter().enumerate() {
                writer.write_documentation(&value.documentations, Some(indentation + 2))?;
                writer.writeln_fmt(
                    format_args!(
                        "{}{}",
                        prefix.clone() + Helper::first_char_uppercase(&value.variant_name).as_str(),
                        if i < enumeration.values.len() - 1 {
                            ", "
                        } else {
                            ""
                        }
                    ),
                    Some(indentation + 2),
                )?;
            }

            writer.writeln(");", Some(indentation))?;
        } else {
            writer.writeln_fmt(
                format_args!(
                    "{} = ({});",
                    Helper::as_type_name(&enumeration.name, &options.type_prefix),
                    enumeration
                        .values
                        .iter()
                        .map(|v| prefix.clone()
                            + Helper::first_char_uppercase(&v.variant_name).as_str())
                        .collect::<Vec<String>>()
                        .join(", ")
                ),
                Some(indentation),
            )?;
        }

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
            format_args!("{formatted_enum_name}Helper = record helper for {formatted_enum_name}",),
            Some(indentation),
        )?;

        if options.generate_from_xml {
            writer.writeln_fmt(
                format_args!(
                    "class function FromXmlValue(const pXmlValue: String): {formatted_enum_name}; static;"
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
            Self::generate_helper_to_xml(writer, enumeration, formatted_enum_name.as_str())?;
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
                "class function {formatted_enum_name}Helper.FromXmlValue(const pXmlValue: String): {formatted_enum_name};"
            ),
            None,
        )?;
        writer.writeln("begin", None)?;
        let prefix = Helper::get_enum_variant_prefix(&enumeration.name);

        for (i, value) in enumeration.values.iter().enumerate() {
            writer.writeln_fmt(
                format_args!("if pXmlValue = '{}' then begin", value.xml_value),
                if i == 0 { Some(2) } else { None },
            )?;
            writer.writeln_fmt(
                format_args!(
                    "Result := {}.{}{};",
                    formatted_enum_name,
                    prefix,
                    Helper::first_char_uppercase(&value.variant_name),
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
                "raise Exception.Create('\"' + pXmlValue + '\" is a unknown value for {formatted_enum_name}');"
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
        formatted_enum_name: &str,
    ) -> Result<(), CodeGenError> {
        let max_variant_len = enumeration
            .values
            .iter()
            .map(|v| v.variant_name.len())
            .max()
            .unwrap_or(1);

        writer.writeln_fmt(
            format_args!("function {formatted_enum_name}Helper.ToXmlValue: String;"),
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
                    Helper::first_char_uppercase(&value.variant_name),
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
    use pretty_assertions::assert_eq;
    use std::io::BufWriter;

    use crate::generator::types::EnumerationValue;

    use super::*;

    fn get_test_enumerations() -> Vec<Enumeration> {
        vec![
            Enumeration {
                name: "ItemStatus".to_owned(),
                qualified_name: "ItemStatus".to_owned(),
                documentations: vec![],
                values: vec![
                    EnumerationValue {
                        variant_name: "open".to_owned(),
                        xml_value: "o".to_owned(),
                        documentations: vec![],
                    },
                    EnumerationValue {
                        variant_name: "closed".to_owned(),
                        xml_value: "c".to_owned(),
                        documentations: vec![],
                    },
                    EnumerationValue {
                        variant_name: "unknown".to_owned(),
                        xml_value: "u".to_owned(),
                        documentations: vec![],
                    },
                ],
            },
            Enumeration {
                name: "Fontsize".to_owned(),
                qualified_name: "Fontsize".to_owned(),
                documentations: vec![],
                values: vec![
                    EnumerationValue {
                        variant_name: "small".to_owned(),
                        xml_value: "s".to_owned(),
                        documentations: vec![],
                    },
                    EnumerationValue {
                        variant_name: "medium".to_owned(),
                        xml_value: "m".to_owned(),
                        documentations: vec![],
                    },
                    EnumerationValue {
                        variant_name: "large".to_owned(),
                        xml_value: "l".to_owned(),
                        documentations: vec![],
                    },
                ],
            },
        ]
    }

    #[test]
    fn write_nothing_when_no_enumerations_available() {
        let enumerations = vec![];
        let options = CodeGenOptions::default();
        let buffer = BufWriter::new(Vec::new());
        let mut writer = CodeWriter { buffer };
        EnumCodeGenerator::write_declarations(&mut writer, &enumerations, &options, 0).unwrap();

        let bytes = writer.get_writer().unwrap().clone();
        let content = String::from_utf8(bytes).unwrap();

        assert_eq!(content, "");
    }

    #[test]
    fn write_declarations_all() {
        let enumerations = get_test_enumerations();
        let options = CodeGenOptions {
            generate_from_xml: true,
            generate_to_xml: true,
            ..Default::default()
        };
        let buffer = BufWriter::new(Vec::new());
        let mut writer = CodeWriter { buffer };
        EnumCodeGenerator::write_declarations(&mut writer, &enumerations, &options, 2).unwrap();

        let bytes = writer.get_writer().unwrap().clone();
        let content = String::from_utf8(bytes).unwrap();

        let expected = "  {$REGION 'Enumerations'}\n  \
              // XML Qualified Name: ItemStatus\n  \
              TItemStatus = (isOpen, isClosed, isUnknown);\n\
              \n  \
              // XML Qualified Name: Fontsize\n  \
              TFontsize = (fSmall, fMedium, fLarge);\n  \
              {$ENDREGION}\n\
              \n  \
              {$REGION 'Enumerations Helper'}\n  \
              TItemStatusHelper = record helper for TItemStatus\n    \
                class function FromXmlValue(const pXmlValue: String): TItemStatus; static;\n    \
                function ToXmlValue: String;\n  \
              end;\n\
              \n  \
              TFontsizeHelper = record helper for TFontsize\n    \
                class function FromXmlValue(const pXmlValue: String): TFontsize; static;\n    \
                function ToXmlValue: String;\n  \
              end;\n  \
              {$ENDREGION}\n";

        assert_eq!(content, expected);
    }

    #[test]
    fn write_implementations_all() {
        let enumerations = get_test_enumerations();
        let options = CodeGenOptions {
            generate_from_xml: true,
            generate_to_xml: true,
            ..Default::default()
        };
        let buffer = BufWriter::new(Vec::new());
        let mut writer = CodeWriter { buffer };
        EnumCodeGenerator::write_implementation(&mut writer, &enumerations, &options).unwrap();

        let bytes = writer.get_writer().unwrap().clone();
        let content = String::from_utf8(bytes).unwrap();

        let expected = "{$REGION 'Enumerations Helper'}\n\
              class function TItemStatusHelper.FromXmlValue(const pXmlValue: String): TItemStatus;\n\
              begin\n  \
                if pXmlValue = 'o' then begin\n    \
                  Result := TItemStatus.isOpen;\n  \
                end else if pXmlValue = 'c' then begin\n    \
                  Result := TItemStatus.isClosed;\n  \
                end else if pXmlValue = 'u' then begin\n    \
                  Result := TItemStatus.isUnknown;\n  \
                end else begin\n    \
                  raise Exception.Create('\"' + pXmlValue + '\" is a unknown value for TItemStatus');\n  \
                end;\n\
              end;\n\
              \n\
              function TItemStatusHelper.ToXmlValue: String;\n\
              begin\n  \
                case Self of\n    \
                  TItemStatus.isOpen    : Result := 'o';\n    \
                  TItemStatus.isClosed  : Result := 'c';\n    \
                  TItemStatus.isUnknown : Result := 'u';\n  \
                end;\n\
              end;\n\
              \n\
              class function TFontsizeHelper.FromXmlValue(const pXmlValue: String): TFontsize;\n\
              begin\n  \
                if pXmlValue = 's' then begin\n    \
                  Result := TFontsize.fSmall;\n  \
                end else if pXmlValue = 'm' then begin\n    \
                  Result := TFontsize.fMedium;\n  \
                end else if pXmlValue = 'l' then begin\n    \
                  Result := TFontsize.fLarge;\n  \
                end else begin\n    \
                  raise Exception.Create('\"' + pXmlValue + '\" is a unknown value for TFontsize');\n  \
                end;\n\
              end;\n\
              \n\
              function TFontsizeHelper.ToXmlValue: String;\n\
              begin\n  \
                case Self of\n    \
                  TFontsize.fSmall  : Result := 's';\n    \
                  TFontsize.fMedium : Result := 'm';\n    \
                  TFontsize.fLarge  : Result := 'l';\n  \
                end;\n\
              end;\n\
              {$ENDREGION}\n";

        assert_eq!(content, expected);
    }
}
