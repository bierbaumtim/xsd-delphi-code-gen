use std::io::{BufWriter, Write};

use crate::generator::{
    code_generator_trait::CodeGenOptions,
    internal_representation::DOCUMENT_NAME,
    types::{BinaryEncoding, ClassType, DataType, TypeAlias},
};

use super::helper::Helper;

pub(crate) struct ClassCodeGenerator;

impl ClassCodeGenerator {
    pub(crate) fn write_forward_declerations(
        buffer: &mut BufWriter<Box<dyn Write>>,
        classes: &Vec<ClassType>,
        indentation: usize,
    ) -> Result<(), std::io::Error> {
        buffer.write_all(b"  {$REGION 'Forward Declarations}\n")?;
        for class_type in classes {
            if class_type.name == DOCUMENT_NAME {
                continue;
            }

            buffer.write_fmt(format_args!(
                "{}T{} = class;\n",
                " ".repeat(indentation),
                Helper::first_char_uppercase(&class_type.name),
            ))?;
        }
        buffer.write_all(b"  {$ENDREGION}\n")?;

        Ok(())
    }

    pub(crate) fn write_declarations(
        buffer: &mut BufWriter<Box<dyn Write>>,
        classes: &Vec<ClassType>,
        document: &ClassType,
        options: &CodeGenOptions,
        indentation: usize,
    ) -> Result<(), std::io::Error> {
        buffer.write_all(b"  {$REGION 'Declarations}\n")?;

        Self::generate_class_declaration(buffer, document, options, indentation)?;

        for class_type in classes {
            if class_type.name == DOCUMENT_NAME {
                continue;
            }

            Self::generate_class_declaration(buffer, class_type, options, indentation)?;
        }
        buffer.write_all(b"  {$ENDREGION}\n")?;

        Ok(())
    }

    pub(crate) fn write_implementations(
        buffer: &mut BufWriter<Box<dyn Write>>,
        classes: &Vec<ClassType>,
        document: &ClassType,
        type_aliases: &Vec<TypeAlias>,
        options: &CodeGenOptions,
    ) -> Result<(), std::io::Error> {
        buffer.write_all(b"{$REGION 'Classes'}\n")?;

        Self::generate_class_implementation(buffer, document, type_aliases, options)?;

        buffer.write_all(b"\n")?;

        for (i, class_type) in classes.iter().enumerate() {
            Self::generate_class_implementation(buffer, class_type, type_aliases, options)?;

            if i < classes.len() - 1 {
                buffer.write_all(b"\n")?;
            }
        }
        buffer.write_all(b"{$ENDREGION}\n")?;

        Ok(())
    }

    fn generate_class_declaration(
        buffer: &mut BufWriter<Box<dyn Write>>,
        class_type: &ClassType,
        options: &CodeGenOptions,
        indentation: usize,
    ) -> Result<(), std::io::Error> {
        buffer.write_fmt(format_args!(
            "{}T{} = class{}",
            " ".repeat(indentation),
            Helper::first_char_uppercase(&class_type.name),
            class_type.super_type.as_ref().map_or_else(
                || "(TObject)".to_owned(),
                |v| format!("(T{})", Helper::first_char_uppercase(&v))
            )
        ))?;
        buffer.write_all(b"\n")?;
        buffer.write_fmt(format_args!("{}public\n", " ".repeat(indentation)))?;

        // constructors and destructors
        if options.generate_from_xml {
            buffer.write_fmt(format_args!(
                "{}constructor FromXml(node: IXMLNode);\n",
                " ".repeat(indentation + 2),
            ))?;
        }

        if class_type.variables.iter().any(|v| v.requires_free) {
            buffer.write_fmt(format_args!(
                "{}destructor Destroy; override;\n",
                " ".repeat(indentation + 2),
            ))?;
        }

        if options.generate_to_xml {
            buffer.write_all(b"\n")?;
            buffer.write_fmt(format_args!(
                "{}procedure AppendToXmlRaw(pParent: IXMLNode);\n",
                " ".repeat(indentation + 2),
            ))?;
            // TODO: Introduce GeneratorOptions
            // file.write_fmt(format_args!(
            //     "{}function ToXml: String;\n",
            //     " ".repeat(indentation + 2),
            // ))?;
            buffer.write_all(b"\n")?;
        }

        // Variables
        for variable in &class_type.variables {
            buffer.write_fmt(format_args!(
                "{}{}: {};\n",
                " ".repeat(indentation + 2),
                Helper::first_char_uppercase(&variable.name),
                Helper::get_datatype_language_representation(&variable.data_type),
            ))?;
        }

        buffer.write_fmt(format_args!("{}end;\n\n", " ".repeat(indentation)))?;

        Ok(())
    }

    fn generate_class_implementation(
        buffer: &mut BufWriter<Box<dyn Write>>,
        class_type: &ClassType,
        type_aliases: &Vec<TypeAlias>,
        options: &CodeGenOptions,
    ) -> Result<(), std::io::Error> {
        let formated_name = Helper::as_type_name(&class_type.name);
        let needs_destroy = class_type.variables.iter().any(|v| v.requires_free);

        buffer.write_fmt(format_args!("{{ {} }}\n", formated_name))?;

        if options.generate_from_xml {
            Self::generate_from_xml_implementation(
                buffer,
                &formated_name,
                class_type,
                type_aliases,
            )?;
        }

        if needs_destroy {
            buffer.write_all(b"\n")?;
            buffer.write_fmt(format_args!("destructor {}.Destroy;\n", formated_name))?;

            buffer.write_all(b"begin\n")?;

            for variable in class_type.variables.iter().filter(|v| v.requires_free) {
                buffer.write_fmt(format_args!(
                    "  {}.Free;\n",
                    Helper::first_char_uppercase(&variable.name)
                ))?;
            }

            buffer.write_all(b"\n")?;
            buffer.write_all(b"  inherited;\n")?;
            buffer.write_all(b"end;\n")?;
        }

        if options.generate_to_xml {
            buffer.write_all(b"\n")?;
            Self::generate_to_xml_implementation(buffer, formated_name, class_type, type_aliases)?;
        }

        Ok(())
    }

    fn generate_from_xml_implementation(
        buffer: &mut BufWriter<Box<dyn Write>>,
        formated_name: &String,
        class_type: &ClassType,
        type_aliases: &Vec<TypeAlias>,
    ) -> Result<(), std::io::Error> {
        buffer.write_fmt(format_args!(
            "constructor {}.FromXml(node: IXMLNode);\n",
            formated_name,
        ))?;
        buffer.write_all(b"begin\n")?;

        for variable in &class_type.variables {
            match &variable.data_type {
                DataType::Enumeration(name) => {
                    buffer.write_fmt(format_args!(
                        "  {} := {}Helper.FromXmlValue(node.ChildNodes['{}']);\n",
                        Helper::first_char_uppercase(&variable.name),
                        Helper::as_type_name(name),
                        variable.name
                    ))?;
                }
                DataType::Alias(name) => {
                    let type_alias = type_aliases.iter().find(|t| t.name == name.as_str());

                    if let Some(t) = type_alias {
                        let mut pattern = t.pattern.clone();
                        let mut data_type = t.for_type.clone();

                        loop {
                            match &data_type {
                                DataType::Custom(n) => {
                                    let type_alias =
                                        type_aliases.iter().find(|t| t.name == n.as_str());

                                    if let Some(alias) = type_alias {
                                        if pattern.is_none() {
                                            pattern = alias.pattern.clone();
                                        }

                                        data_type = alias.for_type.clone();
                                    } else {
                                        break;
                                    }
                                }
                                _ => break,
                            }
                        }

                        buffer.write_all(
                            Self::generate_standard_type_from_xml(
                                &data_type,
                                &variable.name,
                                pattern,
                            )
                            .as_bytes(),
                        )?;
                    }
                }
                DataType::Custom(name) => {
                    buffer.write_fmt(format_args!(
                        "  {} := {}.FromXml(node.ChildNodes['{}']);\n",
                        Helper::first_char_uppercase(&variable.name),
                        Helper::as_type_name(name),
                        variable.name
                    ))?;
                }
                DataType::List(_) => todo!(),
                _ => {
                    buffer.write_all(
                        Self::generate_standard_type_from_xml(
                            &variable.data_type,
                            &variable.name,
                            None,
                        )
                        .as_bytes(),
                    )?;
                }
            }
        }
        buffer.write_all(b"end;\n")?;
        Ok(())
    }

    fn generate_to_xml_implementation(
        buffer: &mut BufWriter<Box<dyn Write>>,
        formated_name: String,
        class_type: &ClassType,
        type_aliases: &Vec<TypeAlias>,
    ) -> Result<(), std::io::Error> {
        buffer.write_fmt(format_args!(
            "procedure {}.AppendToXmlRaw(pParent: IXMLNode);\n",
            formated_name
        ))?;
        buffer.write_all(b"begin\n")?;
        buffer.write_all(b"  var node: IXMLNode;\n")?;
        buffer.write_all(b"\n")?;
        for variable in &class_type.variables {
            match &variable.data_type {
                DataType::Enumeration(name) => {
                    buffer.write_fmt(format_args!(
                        "  node := parent.AddChild('{}');\n",
                        variable.name,
                    ))?;

                    buffer.write_fmt(format_args!(
                        "  node.Text := {}ToXmlValue;\n",
                        Helper::as_type_name(name),
                    ))?;
                }
                DataType::Alias(name) => {
                    let type_alias = type_aliases.iter().find(|t| t.name == name.as_str());

                    if let Some(t) = type_alias {
                        let mut pattern = t.pattern.clone();
                        let mut data_type = t.for_type.clone();

                        loop {
                            match &data_type {
                                DataType::Custom(n) => {
                                    let type_alias =
                                        type_aliases.iter().find(|t| t.name == n.as_str());

                                    if let Some(alias) = type_alias {
                                        if pattern.is_none() {
                                            pattern = alias.pattern.clone();
                                        }

                                        data_type = alias.for_type.clone();
                                    } else {
                                        break;
                                    }
                                }
                                _ => break,
                            }
                        }

                        for arg in
                            Self::generate_standard_type_to_xml(&data_type, &variable.name, pattern)
                        {
                            buffer.write_all(arg.as_bytes())?;
                        }
                    }
                }
                DataType::Custom(name) => {
                    buffer.write_fmt(format_args!(
                        "  node := parent.AddChild('{}');\n",
                        variable.name,
                    ))?;
                    buffer.write_fmt(format_args!(
                        "  {}.AppendToXmlRaw(node);\n",
                        Helper::as_type_name(name),
                    ))?;
                }
                DataType::List(_) => todo!(),
                _ => {
                    for arg in Self::generate_standard_type_to_xml(
                        &variable.data_type,
                        &variable.name,
                        None,
                    ) {
                        buffer.write_all(arg.as_bytes())?;
                    }
                }
            }

            buffer.write_all(b"\n")?;
        }
        buffer.write_all(b"end;\n")?;
        Ok(())
    }

    fn generate_standard_type_from_xml<'a>(
        data_type: &'a DataType,
        variable_name: &'a String,
        pattern: Option<String>,
    ) -> String {
        match data_type {
            DataType::Boolean => format!(
                    "  {} := (node.ChildNodes['{}'].Text = 'true') or (node.ChildNodes['{}'].Text = '1');\n",
                    Helper::first_char_uppercase(variable_name),
                    variable_name,variable_name
                ),
                DataType::DateTime | DataType::Date if pattern.is_some() => format!(
                        "  {} := DecodeDateTime(node.ChildNodes['{}'].Text, '{}');\n",
                        Helper::first_char_uppercase(variable_name),
                        variable_name,
                        pattern.unwrap_or_default(),
                    ),
                DataType::DateTime | DataType::Date => format!(
                        "  {} := ISO8601ToDate(node.ChildNodes['{}'].Text);\n",
                        Helper::first_char_uppercase(variable_name),
                        variable_name,
                    ),
            DataType::Double =>format!(
                    "  {} := StrToFloat(node.ChildNodes['{}'].Text);\n",
                    Helper::first_char_uppercase(variable_name),
                    variable_name
                ),
            DataType::Binary(BinaryEncoding::Base64) => format!(
                    "  {} := TNetEncoding.Base64.DecodeStringToBytes(node.ChildNodes['{}'].Text);\n",
                    Helper::first_char_uppercase(variable_name),
                    variable_name
                ),
            DataType::Binary(BinaryEncoding::Hex) => format!(
                    "  HexToBin(node.ChildNodes['{}'].Text, 0, {}, 0, Length(node.ChildNodes['{}'].Text));\n",
                    variable_name,
                    Helper::first_char_uppercase(variable_name),
                    variable_name,
                ),
            DataType::Integer => format!(
                    "  {} := StrToInt(node.ChildNodes['{}'].Text);\n",
                    Helper::first_char_uppercase(variable_name),
                    variable_name
                ),
            DataType::String => format!(
                    "  {} := node.ChildNodes['{}'].Text;\n",
                    Helper::first_char_uppercase(variable_name),
                    variable_name
                ),
                DataType::Time if pattern.is_some() =>format!(
                            "  {} := TimeOf(DecodeDateTime(node.ChildNodes['{}'].Text, '{}'));\n",
                            Helper::first_char_uppercase(variable_name),
                            variable_name,
                            pattern.unwrap_or_default(),
                        ),
            DataType::Time => format!(
                    "  {} := TimeOf(ISO8601ToDate(node.ChildNodes['{}'].Text));\n",
                    Helper::first_char_uppercase(variable_name),
                    variable_name
                ),
            _ => format!(""),
        }
    }

    fn generate_standard_type_to_xml<'a>(
        data_type: &'a DataType,
        variable_name: &'a String,
        pattern: Option<String>,
    ) -> Vec<String> {
        match data_type {
            DataType::Boolean => vec![
                format!("  node := parent.AddChild('{}');\n", variable_name),
                format!(
                    "  node.Text := IfThen({}, 'true', 'false');\n",
                    Helper::first_char_uppercase(variable_name),
                ),
            ],
            DataType::DateTime | DataType::Date if pattern.is_some() => vec![
                format!("  node := parent.AddChild('{}');\n", variable_name),
                format!(
                    "  node.Text := FormatDateTime('{}', {});\n",
                    pattern.unwrap_or_default(),
                    Helper::first_char_uppercase(&variable_name),
                ),
            ],
            DataType::DateTime | DataType::Date => vec![
                format!("  node := parent.AddChild('{}');\n", variable_name,),
                format!(
                    "  node.Text := ISO8601ToDate({});\n",
                    Helper::first_char_uppercase(&variable_name),
                ),
            ],
            DataType::Double => vec![
                format!("  node := parent.AddChild('{}');\n", variable_name,),
                format!(
                    "  node.Text := FloatToStr({});\n",
                    Helper::first_char_uppercase(&variable_name),
                ),
            ],
            DataType::Binary(BinaryEncoding::Base64) => vec![
                format!("  node := parent.AddChild('{}');\n", variable_name,),
                format!(
                    "  node.Text := TNetEncoding.Base64.EncodeStringToBytes({});\n",
                    Helper::first_char_uppercase(&variable_name),
                ),
            ],
            DataType::Binary(BinaryEncoding::Hex) => vec![
                format!("  node := parent.AddChild('{}');\n", variable_name,),
                format!(
                    "  node.Text := BinToHexStr({});\n",
                    Helper::first_char_uppercase(&variable_name),
                ),
            ],
            DataType::Integer => vec![
                format!("  node := parent.AddChild('{}');\n", variable_name,),
                format!(
                    "  node.Text := IntToStr({});\n",
                    Helper::first_char_uppercase(&variable_name),
                ),
            ],
            DataType::String => vec![
                format!("  node := parent.AddChild('{}');\n", variable_name,),
                format!(
                    "  node.Text := {};\n",
                    Helper::first_char_uppercase(&variable_name),
                ),
            ],
            DataType::Time if pattern.is_some() => vec![
                format!("  node := parent.AddChild('{}');\n", variable_name,),
                format!(
                    "  node.Text := EncodeTime({}, '{}');\n",
                    Helper::first_char_uppercase(&variable_name),
                    pattern.unwrap_or_default(),
                ),
            ],
            DataType::Time => vec![
                format!("  node := parent.AddChild('{}');\n", variable_name,),
                format!(
                    "  node.Text := TimeToStr({});\n",
                    Helper::first_char_uppercase(&variable_name),
                ),
            ],
            _ => vec![],
        }
    }
}
