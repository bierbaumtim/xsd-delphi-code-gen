use std::io::{BufWriter, Write};

use crate::generator::{
    code_generator_trait::CodeGenOptions,
    internal_representation::DOCUMENT_NAME,
    types::{BinaryEncoding, ClassType, DataType, TypeAlias, Variable},
};

use super::helper::Helper;

pub(crate) struct ClassCodeGenerator;

impl ClassCodeGenerator {
    pub(crate) fn write_forward_declerations<T: Write>(
        buffer: &mut BufWriter<T>,
        classes: &Vec<ClassType>,
        options: &CodeGenOptions,
        indentation: usize,
    ) -> Result<(), std::io::Error> {
        buffer.write_all(b"  {$REGION 'Forward Declarations}\n")?;
        for class_type in classes {
            if class_type.name == DOCUMENT_NAME {
                continue;
            }

            buffer.write_fmt(format_args!(
                "{}{} = class;\n",
                " ".repeat(indentation),
                Helper::as_type_name(&class_type.name, &options.type_prefix),
            ))?;
        }
        buffer.write_all(b"  {$ENDREGION}\n")?;

        Ok(())
    }

    pub(crate) fn write_declarations<T: Write>(
        buffer: &mut BufWriter<T>,
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

    pub(crate) fn write_implementations<T: Write>(
        buffer: &mut BufWriter<T>,
        classes: &Vec<ClassType>,
        document: &ClassType,
        type_aliases: &[TypeAlias],
        options: &CodeGenOptions,
    ) -> Result<(), std::io::Error> {
        buffer.write_all(b"{$REGION 'Classes'}\n")?;

        Self::generate_class_implementation(buffer, document, type_aliases, options)?;

        buffer.write_all(b"\n")?;

        for (i, class_type) in classes.iter().enumerate() {
            if class_type.name == DOCUMENT_NAME {
                continue;
            }

            Self::generate_class_implementation(buffer, class_type, type_aliases, options)?;

            if i < classes.len() - 1 {
                buffer.write_all(b"\n")?;
            }
        }
        buffer.write_all(b"{$ENDREGION}\n")?;

        Ok(())
    }

    fn generate_class_declaration<T: Write>(
        buffer: &mut BufWriter<T>,
        class_type: &ClassType,
        options: &CodeGenOptions,
        indentation: usize,
    ) -> Result<(), std::io::Error> {
        buffer.write_fmt(format_args!(
            "{}{} = class{}",
            " ".repeat(indentation),
            Helper::as_type_name(&class_type.name, &options.type_prefix),
            class_type.super_type.as_ref().map_or_else(
                || "(TObject)".to_owned(),
                |v| format!("({})", Helper::as_type_name(v, &options.type_prefix))
            )
        ))?;
        buffer.write_all(b"\n")?;
        buffer.write_fmt(format_args!("{}public\n", " ".repeat(indentation)))?;

        // Variables
        for variable in &class_type.variables {
            match &variable.data_type {
                DataType::List(_) => {
                    buffer.write_fmt(format_args!(
                        "{}{}: {};\n",
                        " ".repeat(indentation + 2),
                        Helper::as_variable_name(&variable.name),
                        Helper::get_datatype_language_representation(
                            &variable.data_type,
                            &options.type_prefix
                        ),
                    ))?;
                }
                DataType::FixedSizeList(item_type, size) => {
                    for i in 1..size + 1 {
                        buffer.write_fmt(format_args!(
                            "{}{}{}: {};\n",
                            " ".repeat(indentation + 2),
                            Helper::as_variable_name(&variable.name),
                            i,
                            Helper::get_datatype_language_representation(
                                item_type,
                                &options.type_prefix
                            ),
                        ))?;
                    }
                }
                _ => {
                    buffer.write_fmt(format_args!(
                        "{}{}: {};\n",
                        " ".repeat(indentation + 2),
                        Helper::as_variable_name(&variable.name),
                        Helper::get_datatype_language_representation(
                            &variable.data_type,
                            &options.type_prefix
                        ),
                    ))?;
                }
            }
        }

        buffer.write_all(b"\n")?;

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

            if class_type.name == DOCUMENT_NAME {
                buffer.write_all(b"\n")?;
                buffer.write_fmt(format_args!(
                    "{}function ToXml: String;\n",
                    " ".repeat(indentation + 2),
                ))?;
            }
        }

        buffer.write_fmt(format_args!("{}end;\n\n", " ".repeat(indentation)))?;

        Ok(())
    }

    fn generate_class_implementation<T: Write>(
        buffer: &mut BufWriter<T>,
        class_type: &ClassType,
        type_aliases: &[TypeAlias],
        options: &CodeGenOptions,
    ) -> Result<(), std::io::Error> {
        let formated_name = Helper::as_type_name(&class_type.name, &options.type_prefix);
        let needs_destroy = class_type.variables.iter().any(|v| v.requires_free);

        buffer.write_fmt(format_args!("{{ {} }}\n", formated_name))?;

        if options.generate_from_xml {
            Self::generate_from_xml_implementation(
                buffer,
                &formated_name,
                class_type,
                type_aliases,
                options,
            )?;
        }

        if options.generate_to_xml {
            buffer.write_all(b"\n")?;
            Self::generate_to_xml_implementation(buffer, &formated_name, class_type, type_aliases)?;

            if class_type.name == DOCUMENT_NAME {
                buffer.write_all(b"\n")?;
                Self::generate_document_to_xml_implementation(buffer, &formated_name)?;
            }
        }

        if needs_destroy {
            buffer.write_all(b"\n")?;
            buffer.write_fmt(format_args!("destructor {}.Destroy;\n", formated_name))?;

            buffer.write_all(b"begin\n")?;

            for variable in class_type.variables.iter().filter(|v| v.requires_free) {
                match &variable.data_type {
                    DataType::FixedSizeList(_, size) => {
                        for i in 1..size + 1 {
                            buffer.write_fmt(format_args!(
                                "  {}{}.Free;\n",
                                Helper::as_variable_name(&variable.name),
                                i,
                            ))?;
                        }
                    }
                    _ => {
                        buffer.write_fmt(format_args!(
                            "  {}.Free;\n",
                            Helper::as_variable_name(&variable.name)
                        ))?;
                    }
                }
            }

            buffer.write_all(b"\n")?;
            buffer.write_all(b"  inherited;\n")?;
            buffer.write_all(b"end;\n")?;
        }

        Ok(())
    }

    fn generate_from_xml_implementation<T: Write>(
        buffer: &mut BufWriter<T>,
        formated_name: &String,
        class_type: &ClassType,
        type_aliases: &[TypeAlias],
        options: &CodeGenOptions,
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
                        "  {} := {}.FromXmlValue(node.ChildNodes['{}'].Text);\n",
                        Helper::as_variable_name(&variable.name),
                        Helper::as_type_name(name, &options.type_prefix),
                        variable.xml_name
                    ))?;
                }
                DataType::Alias(name) => {
                    if let Some((data_type, pattern)) =
                        Self::get_alias_data_type(name.as_str(), type_aliases)
                    {
                        buffer.write_all(
                            Self::generate_standard_type_from_xml(
                                &data_type,
                                &Helper::as_variable_name(&variable.name),
                                format!("node.ChildNodes['{}']", variable.xml_name),
                                pattern,
                            )
                            .as_bytes(),
                        )?;
                    }
                }
                DataType::Custom(name) => {
                    buffer.write_fmt(format_args!(
                        "  {} := {}.FromXml(node.ChildNodes['{}']);\n",
                        Helper::as_variable_name(&variable.name),
                        Helper::as_type_name(name, &options.type_prefix),
                        variable.xml_name
                    ))?;
                }
                DataType::List(item_type) => {
                    Self::generate_list_from_xml(
                        buffer,
                        type_aliases,
                        options,
                        variable,
                        item_type,
                    )?;
                }
                DataType::FixedSizeList(item_type, size) => {
                    Self::generate_fixed_size_list_from_xml(
                        buffer,
                        type_aliases,
                        options,
                        variable,
                        item_type,
                        size,
                    )?;
                }
                _ => {
                    buffer.write_all(
                        Self::generate_standard_type_from_xml(
                            &variable.data_type,
                            &Helper::as_variable_name(&variable.name),
                            format!("node.ChildNodes['{}']", variable.xml_name),
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

    fn generate_list_from_xml<T: Write>(
        buffer: &mut BufWriter<T>,
        type_aliases: &[TypeAlias],
        options: &CodeGenOptions,
        variable: &Variable,
        item_type: &DataType,
    ) -> Result<(), std::io::Error> {
        let formatted_variable_name = Helper::as_variable_name(&variable.name);

        buffer.write_fmt(format_args!(
            "  {} := {}.Create;\n",
            formatted_variable_name,
            Helper::get_datatype_language_representation(&variable.data_type, &options.type_prefix),
        ))?;
        buffer.write_all(b"\n")?;
        buffer.write_fmt(format_args!(
            "  var __{}Index := node.ChildNodes.IndexOf('{}');\n",
            variable.name, variable.xml_name
        ))?;
        buffer.write_fmt(format_args!(
            "  if __{}Index >= 0 then begin\n",
            variable.name
        ))?;
        buffer.write_fmt(format_args!(
            "    for var I := 0 to node.ChildNodes.Count - __{}Index - 1 do begin\n",
            variable.name
        ))?;
        buffer.write_fmt(format_args!(
            "      var __{}Node := node.ChildNodes[__{}Index + I];\n",
            variable.name, variable.name,
        ))?;
        buffer.write_fmt(format_args!(
            "      if __{}Node.LocalName <> '{}' then continue;\n",
            variable.name, variable.xml_name,
        ))?;

        match item_type {
            DataType::Enumeration(name) => {
                buffer.write_fmt(format_args!(
                    "      {}.Add({}.FromXmlValue(__{}Node.Text));\n",
                    formatted_variable_name,
                    Helper::as_type_name(name, &options.type_prefix),
                    variable.name,
                ))?;
            }
            DataType::Alias(name) => {
                if let Some((data_type, pattern)) =
                    Self::get_alias_data_type(name.as_str(), type_aliases)
                {
                    buffer.write_fmt(format_args!(
                        "      var {}",
                        Self::generate_standard_type_from_xml(
                            &data_type,
                            &Helper::as_variable_name(&variable.name),
                            format!("__{}Node", variable.name),
                            pattern,
                        ),
                    ))?;
                    buffer.write_fmt(format_args!(
                        "      {}.Add(__{});\n",
                        formatted_variable_name, formatted_variable_name
                    ))?;
                }
            }
            DataType::Custom(name) => {
                buffer.write_fmt(format_args!(
                    "      {}.Add({}.FromXml(__{}Node));\n",
                    formatted_variable_name,
                    Helper::as_type_name(name, &options.type_prefix),
                    variable.name,
                ))?;
            }
            _ => {
                buffer.write_fmt(format_args!(
                    "      var {}",
                    Self::generate_standard_type_from_xml(
                        item_type,
                        &format!("__{}", formatted_variable_name),
                        format!("__{}Node", variable.name),
                        None,
                    ),
                ))?;
                buffer.write_fmt(format_args!(
                    "      {}.Add(__{});\n",
                    formatted_variable_name, formatted_variable_name
                ))?;
            }
        }
        buffer.write_fmt(format_args!("    end;\n"))?;
        buffer.write_fmt(format_args!("  end;\n"))?;
        buffer.write_all(b"\n")?;

        Ok(())
    }

    fn generate_fixed_size_list_from_xml<T: Write>(
        buffer: &mut BufWriter<T>,
        type_aliases: &[TypeAlias],
        options: &CodeGenOptions,
        variable: &Variable,
        item_type: &DataType,
        size: &usize,
    ) -> Result<(), std::io::Error> {
        for i in 1..size + 1 {
            buffer.write_fmt(format_args!(
                "  {}{} := Default({});\n",
                Helper::as_variable_name(&variable.name),
                i,
                Helper::get_datatype_language_representation(item_type, &options.type_prefix),
            ))?;
        }
        buffer.write_all(b"\n")?;
        buffer.write_fmt(format_args!(
            "  var __{}Index := node.ChildNodes.IndexOf('{}');\n",
            variable.name, variable.xml_name
        ))?;
        buffer.write_fmt(format_args!(
            "  if __{}Index >= 0 then begin\n",
            variable.name
        ))?;
        buffer.write_fmt(format_args!(
            "    for var I := 0 to {} do begin\n",
            size - 1
        ))?;
        buffer.write_fmt(format_args!(
            "      var __{}Node := node.ChildNodes[__{}Index + I];\n",
            variable.name, variable.name,
        ))?;
        buffer.write_fmt(format_args!(
            "      if __{}Node.LocalName <> '{}' then break;\n",
            variable.name, variable.xml_name,
        ))?;
        buffer.write_all(b"\n")?;
        buffer.write_fmt(format_args!("      case I of\n"))?;
        for i in 1..size + 1 {
            match item_type {
                DataType::Enumeration(name) => {
                    buffer.write_fmt(format_args!(
                        "        {}: {}{} := {}.FromXmlValue(__{}Node.Text);\n",
                        i - 1,
                        Helper::as_variable_name(&variable.name),
                        i,
                        Helper::as_type_name(name, &options.type_prefix),
                        variable.name,
                    ))?;
                }
                DataType::Alias(name) => {
                    if let Some((data_type, pattern)) =
                        Self::get_alias_data_type(name.as_str(), type_aliases)
                    {
                        buffer.write_fmt(format_args!(
                            "        {}: {}",
                            i - 1,
                            Self::generate_standard_type_from_xml(
                                &data_type,
                                &Helper::as_variable_name(&variable.name),
                                format!("__{}Node", variable.name),
                                pattern,
                            ),
                        ))?;
                    }
                }
                DataType::Custom(name) => {
                    buffer.write_fmt(format_args!(
                        "        {}: {}{} := {}.FromXml(__{}Node);\n",
                        i - 1,
                        Helper::as_variable_name(&variable.name),
                        i,
                        Helper::as_type_name(name, &options.type_prefix),
                        variable.name,
                    ))?;
                }
                _ => {
                    buffer.write_fmt(format_args!(
                        "        {}: {}",
                        i - 1,
                        Self::generate_standard_type_from_xml(
                            item_type,
                            &format!("{}{}", Helper::as_variable_name(&variable.name), i,),
                            format!("__{}Node", variable.name),
                            None,
                        ),
                    ))?;
                }
            }
        }
        buffer.write_fmt(format_args!("      end;\n"))?;
        buffer.write_fmt(format_args!("    end;\n"))?;
        buffer.write_fmt(format_args!("  end;\n"))?;
        Ok(())
    }

    fn generate_document_to_xml_implementation<T: Write>(
        buffer: &mut BufWriter<T>,
        formated_name: &String,
    ) -> Result<(), std::io::Error> {
        buffer.write_fmt(format_args!("function {}.ToXml: String;\n", formated_name))?;
        buffer.write_all(b"begin\n")?;
        buffer.write_all(b"  var vXmlDoc := NewXMLDocument;\n")?;
        buffer.write_all(b"\n")?;
        buffer.write_all(b"  AppendToXmlRaw(vXmlDoc.Node);\n")?;
        buffer.write_all(b"\n")?;
        buffer.write_all(b"  vXmlDoc.SaveToXML(Result);\n")?;
        buffer.write_all(b"end;\n")?;

        Ok(())
    }

    fn generate_to_xml_implementation<T: Write>(
        buffer: &mut BufWriter<T>,
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
        for (index, variable) in class_type.variables.iter().enumerate() {
            match &variable.data_type {
                DataType::Enumeration(_) => {
                    buffer.write_fmt(format_args!(
                        "  node := pParent.AddChild('{}');\n",
                        variable.xml_name,
                    ))?;

                    buffer.write_fmt(format_args!(
                        "  node.Text := {}.ToXmlValue;\n",
                        Helper::as_variable_name(&variable.name),
                    ))?;
                }
                DataType::Alias(name) => {
                    if let Some((data_type, pattern)) =
                        Self::get_alias_data_type(name.as_str(), type_aliases)
                    {
                        for arg in Self::generate_standard_type_to_xml(
                            &data_type,
                            &Helper::as_variable_name(&variable.name),
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
                        "  node := pParent.AddChild('{}');\n",
                        variable.xml_name,
                    ))?;
                    buffer.write_fmt(format_args!(
                        "  {}.AppendToXmlRaw(node);\n",
                        Helper::as_variable_name(&variable.name),
                    ))?;
                }
                DataType::List(lt) => {
                    buffer.write_fmt(format_args!(
                        "  for var {} in {} do begin\n",
                        variable.name,
                        Helper::as_variable_name(&variable.name),
                    ))?;
                    Self::generate_list_to_xml(
                        buffer,
                        lt,
                        &Helper::as_variable_name(&variable.name),
                        &variable.xml_name,
                        type_aliases,
                        4,
                    )?;
                    buffer.write_fmt(format_args!("  end;\n"))?;
                }
                DataType::FixedSizeList(item_type, size) => {
                    for i in 1..size + 1 {
                        Self::generate_list_to_xml(
                            buffer,
                            item_type,
                            &(Helper::first_char_uppercase(&variable.name)
                                + i.to_string().as_str()),
                            &variable.xml_name,
                            type_aliases,
                            2,
                        )?;

                        if i < *size {
                            buffer.write_all(b"\n")?;
                        }
                    }
                }
                _ => {
                    for arg in Self::generate_standard_type_to_xml(
                        &variable.data_type,
                        &Helper::as_variable_name(&variable.name),
                        &variable.xml_name,
                        None,
                        2,
                    ) {
                        buffer.write_all(arg.as_bytes())?;
                    }
                }
            }

            if index < class_type.variables.len() - 1 {
                buffer.write_all(b"\n")?;
            }
        }
        buffer.write_all(b"end;\n")?;
        Ok(())
    }

    fn generate_list_to_xml<T: Write>(
        buffer: &mut BufWriter<T>,
        data_type: &DataType,
        variable_name: &String,
        xml_name: &String,
        type_aliases: &[TypeAlias],
        indentation: usize,
    ) -> Result<(), std::io::Error> {
        match data_type {
            DataType::Enumeration(_) => {
                buffer.write_fmt(format_args!(
                    "{}node := pParent.AddChild('{}');\n",
                    " ".repeat(indentation),
                    xml_name
                ))?;

                buffer.write_fmt(format_args!(
                    "{}node.Text := {}.ToXmlValue;\n",
                    " ".repeat(indentation),
                    variable_name,
                ))?;
            }
            DataType::Alias(name) => {
                if let Some((data_type, pattern)) =
                    Self::get_alias_data_type(name.as_str(), type_aliases)
                {
                    // TODO: Fix this
                    for arg in Self::generate_standard_type_to_xml(
                        &data_type,
                        &String::new(),
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
                    "{}node := pParent.AddChild('{}');\n",
                    " ".repeat(indentation),
                    xml_name
                ))?;
                buffer.write_fmt(format_args!(
                    "{}{}.AppendToXmlRaw(node);\n",
                    " ".repeat(indentation),
                    variable_name,
                ))?;
            }
            DataType::List(_) => (),
            _ => {
                for arg in Self::generate_standard_type_to_xml(
                    data_type,
                    variable_name,
                    xml_name,
                    None,
                    indentation,
                ) {
                    buffer.write_all(arg.as_bytes())?;
                }
            }
        }

        Ok(())
    }

    fn generate_standard_type_from_xml(
        data_type: &DataType,
        variable_name: &String,
        node: String,
        pattern: Option<String>,
    ) -> String {
        match data_type {
            DataType::Boolean => format!(
                "  {} := ({}.Text = 'true') or ({}.Text = '1');\n",
                variable_name, node, node
            ),
            DataType::DateTime | DataType::Date if pattern.is_some() => format!(
                "  {} := DecodeDateTime({}.Text, '{}');\n",
                variable_name,
                node,
                pattern.unwrap_or_default(),
            ),
            DataType::DateTime | DataType::Date => {
                format!("  {} := ISO8601ToDate({}.Text);\n", variable_name, node,)
            }
            DataType::Double => format!("  {} := StrToFloat({}.Text);\n", variable_name, node),
            DataType::Binary(BinaryEncoding::Base64) => format!(
                "  {} := TNetEncoding.Base64.DecodeStringToBytes({}.Text);\n",
                variable_name, node
            ),
            DataType::Binary(BinaryEncoding::Hex) => format!(
                "  HexToBin({}.Text, 0, {}, 0, Length({}.Text));\n",
                node, variable_name, node,
            ),
            DataType::String => format!("  {} := {}.Text;\n", variable_name, node),
            DataType::Time if pattern.is_some() => format!(
                "  {} := TimeOf(DecodeDateTime({}.Text, '{}'));\n",
                variable_name,
                node,
                pattern.unwrap_or_default(),
            ),
            DataType::Time => format!(
                "  {} := TimeOf(ISO8601ToDate({}.Text));\n",
                variable_name, node
            ),
            DataType::SmallInteger
            | DataType::ShortInteger
            | DataType::Integer
            | DataType::LongInteger
            | DataType::UnsignedSmallInteger
            | DataType::UnsignedShortInteger
            | DataType::UnsignedInteger
            | DataType::UnsignedLongInteger => {
                format!("  {} := StrToInt({}.Text);\n", variable_name, node)
            }
            _ => String::new(),
        }
    }

    fn generate_standard_type_to_xml(
        data_type: &DataType,
        variable_name: &String,
        xml_name: &String,
        pattern: Option<String>,
        indentation: usize,
    ) -> Vec<String> {
        match data_type {
            DataType::Boolean => vec![
                format!(
                    "{}node := pParent.AddChild('{}');\n",
                    " ".repeat(indentation),
                    xml_name
                ),
                format!(
                    "{}node.Text := IfThen({}, 'true', 'false');\n",
                    " ".repeat(indentation),
                    variable_name,
                ),
            ],
            DataType::DateTime | DataType::Date if pattern.is_some() => vec![
                format!(
                    "{}node := pParent.AddChild('{}');\n",
                    " ".repeat(indentation),
                    xml_name
                ),
                format!(
                    "{}node.Text := FormatDateTime('{}', {});\n",
                    " ".repeat(indentation),
                    pattern.unwrap_or_default(),
                    variable_name,
                ),
            ],
            DataType::DateTime | DataType::Date => vec![
                format!(
                    "{}node := pParent.AddChild('{}');\n",
                    " ".repeat(indentation),
                    xml_name,
                ),
                format!(
                    "{}node.Text := DateToISO8601({});\n",
                    " ".repeat(indentation),
                    variable_name,
                ),
            ],
            DataType::Double => vec![
                format!(
                    "{}node := pParent.AddChild('{}');\n",
                    " ".repeat(indentation),
                    xml_name,
                ),
                format!(
                    "{}node.Text := FloatToStr({});\n",
                    " ".repeat(indentation),
                    variable_name,
                ),
            ],
            DataType::Binary(BinaryEncoding::Base64) => vec![
                format!(
                    "{}node := pParent.AddChild('{}');\n",
                    " ".repeat(indentation),
                    xml_name,
                ),
                format!(
                    "{}node.Text := TNetEncoding.Base64.EncodeStringToBytes({});\n",
                    " ".repeat(indentation),
                    variable_name,
                ),
            ],
            DataType::Binary(BinaryEncoding::Hex) => vec![
                format!(
                    "{}node := pParent.AddChild('{}');\n",
                    " ".repeat(indentation),
                    xml_name,
                ),
                format!(
                    "{}node.Text := BinToHexStr({});\n",
                    " ".repeat(indentation),
                    variable_name,
                ),
            ],
            DataType::String => vec![
                format!(
                    "{}node := pParent.AddChild('{}');\n",
                    " ".repeat(indentation),
                    xml_name,
                ),
                format!(
                    "{}node.Text := {};\n",
                    " ".repeat(indentation),
                    variable_name,
                ),
            ],
            DataType::Time if pattern.is_some() => vec![
                format!(
                    "{}node := pParent.AddChild('{}');\n",
                    " ".repeat(indentation),
                    xml_name,
                ),
                format!(
                    "{}node.Text := EncodeTime({}, '{}');\n",
                    " ".repeat(indentation),
                    variable_name,
                    pattern.unwrap_or_default(),
                ),
            ],
            DataType::Time => vec![
                format!(
                    "{}node := pParent.AddChild('{}');\n",
                    " ".repeat(indentation),
                    xml_name,
                ),
                format!(
                    "{}node.Text := TimeToStr({});\n",
                    " ".repeat(indentation),
                    variable_name,
                ),
            ],
            DataType::SmallInteger
            | DataType::ShortInteger
            | DataType::Integer
            | DataType::LongInteger
            | DataType::UnsignedSmallInteger
            | DataType::UnsignedShortInteger
            | DataType::UnsignedInteger
            | DataType::UnsignedLongInteger => vec![
                format!(
                    "{}node := pParent.AddChild('{}');\n",
                    " ".repeat(indentation),
                    xml_name,
                ),
                format!(
                    "{}node.Text := IntToStr({});\n",
                    " ".repeat(indentation),
                    variable_name,
                ),
            ],
            _ => vec![],
        }
    }

    fn get_alias_data_type(
        alias: &str,
        type_aliases: &[TypeAlias],
    ) -> Option<(DataType, Option<String>)> {
        if let Some(t) = type_aliases.iter().find(|t| t.name == alias) {
            let mut pattern = t.pattern.clone();
            let mut data_type = t.for_type.clone();

            while let DataType::Custom(n) = &data_type {
                if let Some(alias) = type_aliases.iter().find(|t| t.name == n.as_str()) {
                    if pattern.is_none() {
                        pattern = alias.pattern.clone();
                    }

                    data_type = alias.for_type.clone();
                } else {
                    break;
                }
            }

            return Some((data_type, pattern));
        }

        None
    }
}

#[cfg(test)]
mod tests {
    // use super::*;

    // TODO: Write Test
}
