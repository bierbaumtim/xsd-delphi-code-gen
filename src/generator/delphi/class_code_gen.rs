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
        type_aliases: &[TypeAlias],
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
                |v| format!("(T{})", Helper::first_char_uppercase(v))
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
            let mut variable_name = Helper::first_char_uppercase(&variable.name);

            if let DataType::List(_) = variable.data_type {
                variable_name.push_str(&options.plural_suffix);
            }

            buffer.write_fmt(format_args!(
                "{}{}: {};\n",
                " ".repeat(indentation + 2),
                variable_name,
                Helper::get_datatype_language_representation(&variable.data_type),
            ))?;
        }

        buffer.write_fmt(format_args!("{}end;\n\n", " ".repeat(indentation)))?;

        Ok(())
    }

    fn generate_class_implementation(
        buffer: &mut BufWriter<Box<dyn Write>>,
        class_type: &ClassType,
        type_aliases: &[TypeAlias],
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

        if options.generate_to_xml {
            buffer.write_all(b"\n")?;
            Self::generate_to_xml_implementation(buffer, &formated_name, class_type, type_aliases)?;
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

        Ok(())
    }

    fn generate_from_xml_implementation(
        buffer: &mut BufWriter<Box<dyn Write>>,
        formated_name: &String,
        class_type: &ClassType,
        type_aliases: &[TypeAlias],
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
                        variable.xml_name
                    ))?;
                }
                DataType::Alias(name) => {
                    let type_alias = type_aliases.iter().find(|t| t.name == name.as_str());

                    if let Some(t) = type_alias {
                        let mut pattern = t.pattern.clone();
                        let mut data_type = t.for_type.clone();

                        while let DataType::Custom(n) = &data_type {
                            let type_alias = type_aliases.iter().find(|t| t.name == n.as_str());

                            if let Some(alias) = type_alias {
                                if pattern.is_none() {
                                    pattern = alias.pattern.clone();
                                }

                                data_type = alias.for_type.clone();
                            } else {
                                break;
                            }
                        }

                        buffer.write_all(
                            Self::generate_standard_type_from_xml(
                                &data_type,
                                &variable.name,
                                &variable.xml_name,
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
                        variable.xml_name
                    ))?;
                }
                DataType::List(_) => {
                    buffer.write_fmt(format_args!(
                        "  // Not supported because type of {} is a List\n",
                        variable.name
                    ))?;
                }
                _ => {
                    buffer.write_all(
                        Self::generate_standard_type_from_xml(
                            &variable.data_type,
                            &variable.name,
                            &variable.xml_name,
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
        formated_name: &String,
        class_type: &ClassType,
        type_aliases: &[TypeAlias],
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
                DataType::Enumeration(_) => {
                    buffer.write_fmt(format_args!(
                        "  node := parent.AddChild('{}');\n",
                        variable.xml_name,
                    ))?;

                    buffer.write_fmt(format_args!(
                        "  node.Text := {}.ToXmlValue;\n",
                        Helper::first_char_uppercase(&variable.name),
                    ))?;
                }
                DataType::Alias(name) => {
                    let type_alias = type_aliases.iter().find(|t| t.name == name.as_str());

                    if let Some(t) = type_alias {
                        let mut pattern = t.pattern.clone();
                        let mut data_type = t.for_type.clone();

                        while let DataType::Custom(n) = &data_type {
                            let type_alias = type_aliases.iter().find(|t| t.name == n.as_str());

                            if let Some(alias) = type_alias {
                                if pattern.is_none() {
                                    pattern = alias.pattern.clone();
                                }

                                data_type = alias.for_type.clone();
                            } else {
                                break;
                            }
                        }

                        for arg in Self::generate_standard_type_to_xml(
                            &data_type,
                            &variable.xml_name,
                            pattern,
                            2,
                        ) {
                            buffer.write_all(arg.as_bytes())?;
                        }
                    }
                }
                DataType::Custom(_) => {
                    buffer.write_fmt(format_args!(
                        "  node := parent.AddChild('{}');\n",
                        variable.xml_name,
                    ))?;
                    buffer.write_fmt(format_args!(
                        "  {}.AppendToXmlRaw(node);\n",
                        Helper::first_char_uppercase(&variable.name),
                    ))?;
                }
                DataType::List(lt) => {
                    // let loop_var_name =
                    //     Helper::first_char(&variable.name).unwrap_or_else(|| String::from("v"));

                    buffer.write_fmt(format_args!(
                        "  for var {} in {} do begin\n",
                        variable.name,
                        Helper::first_char_uppercase(&variable.name),
                    ))?;
                    Self::generate_list_to_xml(
                        buffer,
                        lt,
                        &variable.name,
                        &variable.xml_name,
                        type_aliases,
                        4,
                    )?;
                    buffer.write_fmt(format_args!("  end;\n"))?;
                }
                _ => {
                    for arg in Self::generate_standard_type_to_xml(
                        &variable.data_type,
                        &variable.xml_name,
                        None,
                        2,
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

    fn generate_list_to_xml(
        buffer: &mut BufWriter<Box<dyn Write>>,
        data_type: &DataType,
        variable_name: &String,
        xml_name: &String,
        type_aliases: &[TypeAlias],
        indentation: usize,
    ) -> Result<(), std::io::Error> {
        match data_type {
            DataType::Enumeration(_) => {
                buffer.write_fmt(format_args!(
                    "{}node := parent.AddChild('{}');\n",
                    " ".repeat(indentation),
                    xml_name
                ))?;

                buffer.write_fmt(format_args!(
                    "{}node.Text := {}.ToXmlValue;\n",
                    " ".repeat(indentation),
                    variable_name
                ))?;
            }
            DataType::Alias(name) => {
                let type_alias = type_aliases.iter().find(|t| t.name == name.as_str());

                if let Some(t) = type_alias {
                    let mut pattern = t.pattern.clone();
                    let mut data_type = t.for_type.clone();

                    while let DataType::Custom(n) = &data_type {
                        let type_alias = type_aliases.iter().find(|t| t.name == n.as_str());

                        if let Some(alias) = type_alias {
                            if pattern.is_none() {
                                pattern = alias.pattern.clone();
                            }

                            data_type = alias.for_type.clone();
                        } else {
                            break;
                        }
                    }

                    for arg in Self::generate_standard_type_to_xml(
                        &data_type,
                        &String::from("v"),
                        pattern,
                        indentation,
                    ) {
                        buffer.write_all(arg.as_bytes())?;
                    }
                }
            }
            DataType::Custom(_) => {
                buffer.write_fmt(format_args!(
                    "{}node := parent.AddChild('{}');\n",
                    " ".repeat(indentation),
                    xml_name
                ))?;
                buffer.write_fmt(format_args!(
                    "{}{}.AppendToXmlRaw(node);\n",
                    " ".repeat(indentation),
                    variable_name
                ))?;
            }
            DataType::List(_) => (),
            _ => {
                for arg in
                    Self::generate_standard_type_to_xml(data_type, variable_name, None, indentation)
                {
                    buffer.write_all(arg.as_bytes())?;
                }
            }
        }

        Ok(())
    }

    fn generate_standard_type_from_xml<'a>(
        data_type: &'a DataType,
        variable_name: &'a String,
        xml_name: &'a String,
        pattern: Option<String>,
    ) -> String {
        match data_type {
            DataType::Boolean => format!(
                    "  {} := (node.ChildNodes['{}'].Text = 'true') or (node.ChildNodes['{}'].Text = '1');\n",
                    Helper::first_char_uppercase(variable_name),
                    xml_name, xml_name
                ),
                DataType::DateTime | DataType::Date if pattern.is_some() => format!(
                        "  {} := DecodeDateTime(node.ChildNodes['{}'].Text, '{}');\n",
                        Helper::first_char_uppercase(variable_name),
                        xml_name,
                        pattern.unwrap_or_default(),
                    ),
                DataType::DateTime | DataType::Date => format!(
                        "  {} := ISO8601ToDate(node.ChildNodes['{}'].Text);\n",
                        Helper::first_char_uppercase(variable_name),
                        xml_name,
                    ),
            DataType::Double =>format!(
                    "  {} := StrToFloat(node.ChildNodes['{}'].Text);\n",
                    Helper::first_char_uppercase(variable_name),
                    xml_name
                ),
            DataType::Binary(BinaryEncoding::Base64) => format!(
                    "  {} := TNetEncoding.Base64.DecodeStringToBytes(node.ChildNodes['{}'].Text);\n",
                    Helper::first_char_uppercase(variable_name),
                    xml_name
                ),
            DataType::Binary(BinaryEncoding::Hex) => format!(
                    "  HexToBin(node.ChildNodes['{}'].Text, 0, {}, 0, Length(node.ChildNodes['{}'].Text));\n",
                    xml_name,
                    Helper::first_char_uppercase(variable_name),
                    xml_name,
                ),
            DataType::Integer => format!(
                    "  {} := StrToInt(node.ChildNodes['{}'].Text);\n",
                    Helper::first_char_uppercase(variable_name),
                    xml_name
                ),
            DataType::String => format!(
                    "  {} := node.ChildNodes['{}'].Text;\n",
                    Helper::first_char_uppercase(variable_name),
                    xml_name
                ),
                DataType::Time if pattern.is_some() =>format!(
                            "  {} := TimeOf(DecodeDateTime(node.ChildNodes['{}'].Text, '{}'));\n",
                            Helper::first_char_uppercase(variable_name),
                            xml_name,
                            pattern.unwrap_or_default(),
                        ),
            DataType::Time => format!(
                    "  {} := TimeOf(ISO8601ToDate(node.ChildNodes['{}'].Text));\n",
                    Helper::first_char_uppercase(variable_name),
                    xml_name
                ),
            _ => String::new(),
        }
    }

    fn generate_standard_type_to_xml<'a>(
        data_type: &'a DataType,
        variable_name: &'a String,
        pattern: Option<String>,
        indentation: usize,
    ) -> Vec<String> {
        match data_type {
            DataType::Boolean => vec![
                format!(
                    "{}node := parent.AddChild('{}');\n",
                    " ".repeat(indentation),
                    variable_name
                ),
                format!(
                    "{}node.Text := IfThen({}, 'true', 'false');\n",
                    " ".repeat(indentation),
                    Helper::first_char_uppercase(variable_name),
                ),
            ],
            DataType::DateTime | DataType::Date if pattern.is_some() => vec![
                format!(
                    "{}node := parent.AddChild('{}');\n",
                    " ".repeat(indentation),
                    variable_name
                ),
                format!(
                    "{}node.Text := FormatDateTime('{}', {});\n",
                    " ".repeat(indentation),
                    pattern.unwrap_or_default(),
                    Helper::first_char_uppercase(variable_name),
                ),
            ],
            DataType::DateTime | DataType::Date => vec![
                format!(
                    "{}node := parent.AddChild('{}');\n",
                    " ".repeat(indentation),
                    variable_name,
                ),
                format!(
                    "{}node.Text := ISO8601ToDate({});\n",
                    " ".repeat(indentation),
                    Helper::first_char_uppercase(variable_name),
                ),
            ],
            DataType::Double => vec![
                format!(
                    "{}node := parent.AddChild('{}');\n",
                    " ".repeat(indentation),
                    variable_name,
                ),
                format!(
                    "{}node.Text := FloatToStr({});\n",
                    " ".repeat(indentation),
                    Helper::first_char_uppercase(variable_name),
                ),
            ],
            DataType::Binary(BinaryEncoding::Base64) => vec![
                format!(
                    "{}node := parent.AddChild('{}');\n",
                    " ".repeat(indentation),
                    variable_name,
                ),
                format!(
                    "{}node.Text := TNetEncoding.Base64.EncodeStringToBytes({});\n",
                    " ".repeat(indentation),
                    Helper::first_char_uppercase(variable_name),
                ),
            ],
            DataType::Binary(BinaryEncoding::Hex) => vec![
                format!(
                    "{}node := parent.AddChild('{}');\n",
                    " ".repeat(indentation),
                    variable_name,
                ),
                format!(
                    "{}node.Text := BinToHexStr({});\n",
                    " ".repeat(indentation),
                    Helper::first_char_uppercase(variable_name),
                ),
            ],
            DataType::Integer => vec![
                format!(
                    "{}node := parent.AddChild('{}');\n",
                    " ".repeat(indentation),
                    variable_name,
                ),
                format!(
                    "{}node.Text := IntToStr({});\n",
                    " ".repeat(indentation),
                    Helper::first_char_uppercase(variable_name),
                ),
            ],
            DataType::String => vec![
                format!(
                    "{}node := parent.AddChild('{}');\n",
                    " ".repeat(indentation),
                    variable_name,
                ),
                format!(
                    "{}node.Text := {};\n",
                    " ".repeat(indentation),
                    Helper::first_char_uppercase(variable_name),
                ),
            ],
            DataType::Time if pattern.is_some() => vec![
                format!(
                    "{}node := parent.AddChild('{}');\n",
                    " ".repeat(indentation),
                    variable_name,
                ),
                format!(
                    "{}node.Text := EncodeTime({}, '{}');\n",
                    " ".repeat(indentation),
                    Helper::first_char_uppercase(variable_name),
                    pattern.unwrap_or_default(),
                ),
            ],
            DataType::Time => vec![
                format!(
                    "{}node := parent.AddChild('{}');\n",
                    " ".repeat(indentation),
                    variable_name,
                ),
                format!(
                    "{}node.Text := TimeToStr({});\n",
                    " ".repeat(indentation),
                    Helper::first_char_uppercase(variable_name),
                ),
            ],
            _ => vec![],
        }
    }
}
