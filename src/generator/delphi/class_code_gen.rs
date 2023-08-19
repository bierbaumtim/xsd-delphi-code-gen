use std::io::Write;

use crate::generator::{
    code_generator_trait::{CodeGenError, CodeGenOptions},
    internal_representation::DOCUMENT_NAME,
    types::{BinaryEncoding, ClassType, DataType, TypeAlias, Variable},
};

use super::{code_writer::CodeWriter, helper::Helper};

pub(crate) struct ClassCodeGenerator;

impl ClassCodeGenerator {
    pub(crate) fn write_forward_declerations<T: Write>(
        writer: &mut CodeWriter<T>,
        classes: &Vec<ClassType>,
        options: &CodeGenOptions,
        indentation: usize,
    ) -> Result<(), std::io::Error> {
        writer.writeln("{$REGION 'Forward Declarations}", Some(indentation))?;
        for class_type in classes {
            if class_type.name == DOCUMENT_NAME {
                continue;
            }

            writer.writeln_fmt(
                format_args!(
                    "{} = class;",
                    Helper::as_type_name(&class_type.name, &options.type_prefix),
                ),
                Some(indentation),
            )?;
        }
        writer.writeln("{$ENDREGION}", Some(indentation))?;

        Ok(())
    }

    pub(crate) fn write_declarations<T: Write>(
        writer: &mut CodeWriter<T>,
        classes: &Vec<ClassType>,
        document: &ClassType,
        options: &CodeGenOptions,
        indentation: usize,
    ) -> Result<(), CodeGenError> {
        writer.writeln("{$REGION 'Declarations}", Some(2))?;

        Self::generate_class_declaration(writer, document, options, indentation)?;

        for class_type in classes {
            if class_type.name == DOCUMENT_NAME {
                continue;
            }

            Self::generate_class_declaration(writer, class_type, options, indentation)?;
        }
        writer.writeln("{$ENDREGION}", Some(2))?;

        Ok(())
    }

    pub(crate) fn write_implementations<T: Write>(
        writer: &mut CodeWriter<T>,
        classes: &Vec<ClassType>,
        document: &ClassType,
        type_aliases: &[TypeAlias],
        options: &CodeGenOptions,
    ) -> Result<(), CodeGenError> {
        writer.writeln("{$REGION 'Classes'}", None)?;

        Self::generate_class_implementation(writer, document, type_aliases, options)?;

        writer.newline()?;

        for (i, class_type) in classes.iter().enumerate() {
            if class_type.name == DOCUMENT_NAME {
                continue;
            }

            Self::generate_class_implementation(writer, class_type, type_aliases, options)?;

            if i < classes.len() - 1 {
                writer.newline()?;
            }
        }
        writer.writeln("{$ENDREGION}", None)?;

        Ok(())
    }

    fn generate_class_declaration<T: Write>(
        writer: &mut CodeWriter<T>,
        class_type: &ClassType,
        options: &CodeGenOptions,
        indentation: usize,
    ) -> Result<(), CodeGenError> {
        writer.write_documentation(&class_type.documentations, Some(indentation))?;
        writer.writeln_fmt(
            format_args!("// XML Qualified Name: {}", class_type.qualified_name),
            Some(indentation),
        )?;
        writer.writeln_fmt(
            format_args!(
                "{} = class{}",
                Helper::as_type_name(&class_type.name, &options.type_prefix),
                class_type.super_type.as_ref().map_or_else(
                    || "(TObject)".to_owned(),
                    |v| format!("({})", Helper::as_type_name(v, &options.type_prefix))
                )
            ),
            Some(indentation),
        )?;
        writer.writeln("public", Some(indentation))?;

        // Variables
        for variable in &class_type.variables {
            match &variable.data_type {
                DataType::List(_) => {
                    writer.writeln_fmt(
                        format_args!(
                            "{}: {};",
                            Helper::as_variable_name(&variable.name),
                            Helper::get_datatype_language_representation(
                                &variable.data_type,
                                &options.type_prefix
                            ),
                        ),
                        Some(indentation + 2),
                    )?;
                }
                DataType::FixedSizeList(item_type, size) => {
                    for i in 1..size + 1 {
                        writer.writeln_fmt(
                            format_args!(
                                "{}{}: {};",
                                Helper::as_variable_name(&variable.name),
                                i,
                                Helper::get_datatype_language_representation(
                                    item_type,
                                    &options.type_prefix
                                ),
                            ),
                            Some(indentation + 2),
                        )?;
                    }
                }
                _ => {
                    writer.writeln_fmt(
                        format_args!(
                            "{}: {};",
                            Helper::as_variable_name(&variable.name),
                            Helper::get_datatype_language_representation(
                                &variable.data_type,
                                &options.type_prefix
                            ),
                        ),
                        Some(indentation + 2),
                    )?;
                }
            }
        }

        writer.newline()?;

        let fn_decorator = class_type
            .super_type
            .as_ref()
            .map_or("virtual", |_| "override");

        // constructors and destructors
        if options.generate_to_xml {
            writer.writeln_fmt(
                format_args!("constructor Create; {};", fn_decorator),
                Some(indentation + 2),
            )?;
        }
        if options.generate_from_xml {
            writer.writeln_fmt(
                format_args!("constructor FromXml(node: IXMLNode); {};", fn_decorator),
                Some(indentation + 2),
            )?;
        }

        if class_type.variables.iter().any(|v| v.requires_free) {
            writer.writeln("destructor Destroy; override;", Some(indentation + 2))?;
        }

        if options.generate_to_xml {
            writer.newline()?;
            writer.writeln_fmt(
                format_args!(
                    "procedure AppendToXmlRaw(pParent: IXMLNode); {};",
                    fn_decorator,
                ),
                Some(indentation + 2),
            )?;

            if class_type.name == DOCUMENT_NAME {
                writer.newline()?;
                writer.writeln("function ToXml: String;", Some(indentation + 2))?;
            }
        }

        writer.writeln("end;", Some(indentation))?;
        writer.newline()?;

        Ok(())
    }

    fn generate_class_implementation<T: Write>(
        writer: &mut CodeWriter<T>,
        class_type: &ClassType,
        type_aliases: &[TypeAlias],
        options: &CodeGenOptions,
    ) -> Result<(), CodeGenError> {
        let formated_name = Helper::as_type_name(&class_type.name, &options.type_prefix);
        let needs_destroy = class_type.variables.iter().any(|v| v.requires_free);

        writer.writeln_fmt(format_args!("{{ {} }}", formated_name), None)?;

        if options.generate_to_xml {
            Self::generate_constructor_implementation(writer, &formated_name, class_type, options)?;
        }

        if options.generate_from_xml {
            if options.generate_to_xml {
                writer.newline()?;
            }

            Self::generate_from_xml_implementation(
                writer,
                &formated_name,
                class_type,
                type_aliases,
                options,
            )?;
        }

        if options.generate_to_xml {
            writer.newline()?;
            Self::generate_to_xml_implementation(writer, &formated_name, class_type, type_aliases)?;

            if class_type.name == DOCUMENT_NAME {
                writer.newline()?;
                Self::generate_document_to_xml_implementation(writer, &formated_name)?;
            }
        }

        if needs_destroy {
            writer.newline()?;
            writer.writeln_fmt(format_args!("destructor {}.Destroy;", formated_name), None)?;

            writer.writeln("begin", None)?;

            for variable in class_type.variables.iter().filter(|v| v.requires_free) {
                match &variable.data_type {
                    DataType::FixedSizeList(_, size) => {
                        for i in 1..size + 1 {
                            writer.writeln_fmt(
                                format_args!(
                                    "{}{}.Free;",
                                    Helper::as_variable_name(&variable.name),
                                    i,
                                ),
                                Some(2),
                            )?;
                        }
                    }
                    _ => {
                        writer.writeln_fmt(
                            format_args!("{}.Free;", Helper::as_variable_name(&variable.name)),
                            Some(2),
                        )?;
                    }
                }
            }

            writer.newline()?;
            writer.writeln("inherited;", Some(2))?;
            writer.writeln("end;", None)?;
        }

        Ok(())
    }

    fn generate_constructor_implementation<T: Write>(
        writer: &mut CodeWriter<T>,
        formated_name: &String,
        class_type: &ClassType,
        options: &CodeGenOptions,
    ) -> Result<(), CodeGenError> {
        writer.writeln_fmt(format_args!("constructor {}.Create;", formated_name), None)?;
        writer.writeln("begin", None)?;

        if class_type.super_type.is_some() {
            writer.writeln("inherited;", Some(2))?;
            writer.newline()?;
        }

        for variable in &class_type.variables {
            match &variable.data_type {
                DataType::Alias(name) => writer.writeln_fmt(
                    format_args!(
                        "{} := Default({});",
                        Helper::as_variable_name(&variable.name),
                        Helper::as_type_name(name, &options.type_prefix),
                    ),
                    Some(2),
                )?,
                DataType::Enumeration(name) => writer.writeln_fmt(
                    format_args!(
                        "{} := Default({});",
                        Helper::as_variable_name(&variable.name),
                        Helper::as_type_name(name, &options.type_prefix),
                    ),
                    Some(2),
                )?,
                DataType::Custom(name) => {
                    writer.writeln_fmt(
                        format_args!(
                            "{} := {}.Create;",
                            Helper::as_variable_name(&variable.name),
                            Helper::as_type_name(name, &options.type_prefix),
                        ),
                        Some(2),
                    )?;
                }
                DataType::List(_) => {
                    writer.writeln_fmt(
                        format_args!(
                            "{} := {}.Create;",
                            Helper::as_variable_name(&variable.name),
                            Helper::get_datatype_language_representation(
                                &variable.data_type,
                                &options.type_prefix
                            ),
                        ),
                        Some(2),
                    )?;
                }
                DataType::FixedSizeList(item_type, size) => {
                    let rhs = match item_type.as_ref() {
                        DataType::Alias(name) => format!(
                            "Default({})",
                            Helper::as_type_name(name, &options.type_prefix)
                        ),
                        DataType::Enumeration(name) => format!(
                            "Default({})",
                            Helper::as_type_name(name, &options.type_prefix)
                        ),
                        DataType::Custom(name) => {
                            format!(
                                "{}.Create",
                                Helper::as_type_name(name, &options.type_prefix)
                            )
                        }
                        DataType::List(_) => {
                            return Err(CodeGenError::NestedListInFixedSizeList(
                                class_type.name.clone(),
                                variable.name.clone(),
                            ))
                        }
                        DataType::FixedSizeList(_, _) => {
                            return Err(CodeGenError::NestedFixedSizeList(
                                class_type.name.clone(),
                                variable.name.clone(),
                            ))
                        }
                        _ => format!(
                            "Default({})",
                            Helper::get_datatype_language_representation(
                                item_type.as_ref(),
                                &options.type_prefix
                            )
                        ),
                    };

                    for i in 1..size + 1 {
                        writer.writeln_fmt(
                            format_args!(
                                "{}{} := {};",
                                Helper::as_variable_name(&variable.name),
                                i,
                                rhs,
                            ),
                            Some(2),
                        )?;
                    }
                }
                _ => {
                    writer.writeln_fmt(
                        format_args!(
                            "{} := Default({});",
                            Helper::as_variable_name(&variable.name),
                            Helper::get_datatype_language_representation(
                                &variable.data_type,
                                &options.type_prefix
                            ),
                        ),
                        Some(2),
                    )?;
                }
            }
        }

        writer.writeln("end;", None)?;

        Ok(())
    }

    fn generate_from_xml_implementation<T: Write>(
        writer: &mut CodeWriter<T>,
        formated_name: &String,
        class_type: &ClassType,
        type_aliases: &[TypeAlias],
        options: &CodeGenOptions,
    ) -> Result<(), CodeGenError> {
        writer.writeln_fmt(
            format_args!("constructor {}.FromXml(node: IXMLNode);", formated_name,),
            None,
        )?;
        writer.writeln("begin", None)?;

        if class_type.super_type.is_some() {
            writer.writeln("inherited;", Some(2))?;
            writer.newline()?;
        }

        for variable in &class_type.variables {
            match &variable.data_type {
                DataType::Enumeration(name) => {
                    writer.writeln_fmt(
                        format_args!(
                            "{} := {}.FromXmlValue(node.ChildNodes['{}'].Text);",
                            Helper::as_variable_name(&variable.name),
                            Helper::as_type_name(name, &options.type_prefix),
                            variable.xml_name
                        ),
                        Some(2),
                    )?;
                }
                DataType::Alias(name) => {
                    if let Some((data_type, pattern)) =
                        Helper::get_alias_data_type(name.as_str(), type_aliases)
                    {
                        writer.writeln(
                            Self::generate_standard_type_from_xml(
                                &data_type,
                                &Helper::as_variable_name(&variable.name),
                                format!("node.ChildNodes['{}']", variable.xml_name),
                                pattern,
                            )
                            .as_str(),
                            Some(2),
                        )?;
                    }
                }
                DataType::Custom(name) => {
                    writer.writeln_fmt(
                        format_args!(
                            "{} := {}.FromXml(node.ChildNodes['{}']);",
                            Helper::as_variable_name(&variable.name),
                            Helper::as_type_name(name, &options.type_prefix),
                            variable.xml_name
                        ),
                        Some(2),
                    )?;
                }
                DataType::List(item_type) => {
                    Self::generate_list_from_xml(
                        writer,
                        type_aliases,
                        options,
                        variable,
                        item_type,
                    )?;
                }
                DataType::FixedSizeList(item_type, size) => {
                    Self::generate_fixed_size_list_from_xml(
                        writer,
                        type_aliases,
                        options,
                        variable,
                        item_type,
                        size,
                    )?;
                }
                _ => {
                    writer.writeln(
                        Self::generate_standard_type_from_xml(
                            &variable.data_type,
                            &Helper::as_variable_name(&variable.name),
                            format!("node.ChildNodes['{}']", variable.xml_name),
                            None,
                        )
                        .as_str(),
                        Some(2),
                    )?;
                }
            }
        }
        writer.writeln("end;", None)?;

        Ok(())
    }

    fn generate_list_from_xml<T: Write>(
        writer: &mut CodeWriter<T>,
        type_aliases: &[TypeAlias],
        options: &CodeGenOptions,
        variable: &Variable,
        item_type: &DataType,
    ) -> Result<(), CodeGenError> {
        let formatted_variable_name = Helper::as_variable_name(&variable.name);

        writer.writeln_fmt(
            format_args!(
                "{} := {}.Create;",
                formatted_variable_name,
                Helper::get_datatype_language_representation(
                    &variable.data_type,
                    &options.type_prefix
                ),
            ),
            Some(2),
        )?;
        writer.newline()?;
        writer.writeln_fmt(
            format_args!(
                "var __{}Index := node.ChildNodes.IndexOf('{}');",
                variable.name, variable.xml_name
            ),
            Some(2),
        )?;
        writer.writeln_fmt(
            format_args!("if __{}Index >= 0 then begin", variable.name),
            Some(2),
        )?;
        writer.writeln_fmt(
            format_args!(
                "for var I := 0 to node.ChildNodes.Count - __{}Index - 1 do begin",
                variable.name
            ),
            Some(4),
        )?;
        writer.writeln_fmt(
            format_args!(
                "var __{}Node := node.ChildNodes[__{}Index + I];",
                variable.name, variable.name,
            ),
            Some(6),
        )?;
        writer.writeln_fmt(
            format_args!(
                "if __{}Node.LocalName <> '{}' then continue;",
                variable.name, variable.xml_name,
            ),
            Some(6),
        )?;

        match item_type {
            DataType::Enumeration(name) => {
                writer.writeln_fmt(
                    format_args!(
                        "{}.Add({}.FromXmlValue(__{}Node.Text));",
                        formatted_variable_name,
                        Helper::as_type_name(name, &options.type_prefix),
                        variable.name,
                    ),
                    Some(6),
                )?;
            }
            DataType::Alias(name) => {
                if let Some((data_type, pattern)) =
                    Helper::get_alias_data_type(name.as_str(), type_aliases)
                {
                    writer.writeln_fmt(
                        format_args!(
                            "var {}",
                            Self::generate_standard_type_from_xml(
                                &data_type,
                                &Helper::as_variable_name(&variable.name),
                                format!("__{}Node", variable.name),
                                pattern,
                            ),
                        ),
                        Some(6),
                    )?;
                    writer.writeln_fmt(
                        format_args!(
                            "{}.Add(__{});",
                            formatted_variable_name, formatted_variable_name
                        ),
                        Some(6),
                    )?;
                }
            }
            DataType::Custom(name) => {
                writer.writeln_fmt(
                    format_args!(
                        "{}.Add({}.FromXml(__{}Node));",
                        formatted_variable_name,
                        Helper::as_type_name(name, &options.type_prefix),
                        variable.name,
                    ),
                    Some(6),
                )?;
            }
            _ => {
                writer.writeln_fmt(
                    format_args!(
                        "var {}",
                        Self::generate_standard_type_from_xml(
                            item_type,
                            &format!("__{}", formatted_variable_name),
                            format!("__{}Node", variable.name),
                            None,
                        ),
                    ),
                    Some(6),
                )?;
                writer.writeln_fmt(
                    format_args!(
                        "{}.Add(__{});",
                        formatted_variable_name, formatted_variable_name
                    ),
                    Some(6),
                )?;
            }
        }
        writer.writeln("end;", Some(4))?;
        writer.writeln("end;", Some(2))?;
        writer.newline()?;

        Ok(())
    }

    fn generate_fixed_size_list_from_xml<T: Write>(
        writer: &mut CodeWriter<T>,
        type_aliases: &[TypeAlias],
        options: &CodeGenOptions,
        variable: &Variable,
        item_type: &DataType,
        size: &usize,
    ) -> Result<(), CodeGenError> {
        for i in 1..size + 1 {
            writer.writeln_fmt(
                format_args!(
                    "{}{} := Default({});",
                    Helper::as_variable_name(&variable.name),
                    i,
                    Helper::get_datatype_language_representation(item_type, &options.type_prefix),
                ),
                Some(4),
            )?;
        }
        writer.newline()?;
        writer.writeln_fmt(
            format_args!(
                "var __{}Index := node.ChildNodes.IndexOf('{}');",
                variable.name, variable.xml_name
            ),
            Some(2),
        )?;
        writer.writeln_fmt(
            format_args!("if __{}Index >= 0 then begin", variable.name),
            Some(2),
        )?;
        writer.writeln_fmt(
            format_args!("for var I := 0 to {} do begin", size - 1),
            Some(4),
        )?;
        writer.writeln_fmt(
            format_args!(
                "var __{}Node := node.ChildNodes[__{}Index + I];",
                variable.name, variable.name,
            ),
            Some(6),
        )?;
        writer.writeln_fmt(
            format_args!(
                "if __{}Node.LocalName <> '{}' then break;",
                variable.name, variable.xml_name,
            ),
            Some(6),
        )?;
        writer.newline()?;
        writer.writeln("case I of", Some(6))?;
        for i in 1..size + 1 {
            match item_type {
                DataType::Enumeration(name) => {
                    writer.writeln_fmt(
                        format_args!(
                            "{}: {}{} := {}.FromXmlValue(__{}Node.Text);",
                            i - 1,
                            Helper::as_variable_name(&variable.name),
                            i,
                            Helper::as_type_name(name, &options.type_prefix),
                            variable.name,
                        ),
                        Some(8),
                    )?;
                }
                DataType::Alias(name) => {
                    if let Some((data_type, pattern)) =
                        Helper::get_alias_data_type(name.as_str(), type_aliases)
                    {
                        writer.writeln_fmt(
                            format_args!(
                                "{}: {}",
                                i - 1,
                                Self::generate_standard_type_from_xml(
                                    &data_type,
                                    &Helper::as_variable_name(&variable.name),
                                    format!("__{}Node", variable.name),
                                    pattern,
                                ),
                            ),
                            Some(8),
                        )?;
                    }
                }
                DataType::Custom(name) => {
                    writer.writeln_fmt(
                        format_args!(
                            "{}: {}{} := {}.FromXml(__{}Node);",
                            i - 1,
                            Helper::as_variable_name(&variable.name),
                            i,
                            Helper::as_type_name(name, &options.type_prefix),
                            variable.name,
                        ),
                        Some(8),
                    )?;
                }
                _ => {
                    writer.writeln_fmt(
                        format_args!(
                            "{}: {}",
                            i - 1,
                            Self::generate_standard_type_from_xml(
                                item_type,
                                &format!("{}{}", Helper::as_variable_name(&variable.name), i,),
                                format!("__{}Node", variable.name),
                                None,
                            ),
                        ),
                        Some(8),
                    )?;
                }
            }
        }
        writer.writeln("end;", Some(6))?;
        writer.writeln("end;", Some(4))?;
        writer.writeln("end;", Some(2))?;
        Ok(())
    }

    fn generate_document_to_xml_implementation<T: Write>(
        writer: &mut CodeWriter<T>,
        formated_name: &String,
    ) -> Result<(), std::io::Error> {
        writer.writeln_fmt(
            format_args!("function {}.ToXml: String;", formated_name),
            None,
        )?;
        writer.writeln("begin", None)?;
        writer.writeln("var vXmlDoc := NewXMLDocument;", Some(2))?;
        writer.newline()?;
        writer.writeln("AppendToXmlRaw(vXmlDoc.Node);", Some(2))?;
        writer.newline()?;
        writer.writeln("vXmlDoc.SaveToXML(Result);", Some(2))?;
        writer.writeln("end;", None)?;

        Ok(())
    }

    fn generate_to_xml_implementation<T: Write>(
        writer: &mut CodeWriter<T>,
        formated_name: &String,
        class_type: &ClassType,
        type_aliases: &[TypeAlias],
    ) -> Result<(), CodeGenError> {
        writer.writeln_fmt(
            format_args!(
                "procedure {}.AppendToXmlRaw(pParent: IXMLNode);",
                formated_name
            ),
            None,
        )?;
        writer.writeln("begin", None)?;

        if class_type.super_type.is_some() {
            writer.writeln("inherited;", Some(2))?;
            writer.newline()?;
        }

        writer.writeln("var node: IXMLNode;", Some(2))?;
        writer.newline()?;
        for (index, variable) in class_type.variables.iter().enumerate() {
            match &variable.data_type {
                DataType::Enumeration(_) => {
                    writer.writeln_fmt(
                        format_args!("node := pParent.AddChild('{}');", variable.xml_name,),
                        Some(2),
                    )?;

                    writer.writeln_fmt(
                        format_args!(
                            "node.Text := {}.ToXmlValue;",
                            Helper::as_variable_name(&variable.name),
                        ),
                        Some(2),
                    )?;
                }
                DataType::Alias(name) => {
                    if let Some((data_type, pattern)) =
                        Helper::get_alias_data_type(name.as_str(), type_aliases)
                    {
                        for arg in Self::generate_standard_type_to_xml(
                            &data_type,
                            &Helper::as_variable_name(&variable.name),
                            &variable.xml_name,
                            pattern,
                        ) {
                            writer.writeln(arg.as_str(), Some(2))?;
                        }
                    }
                }
                DataType::Custom(_) => {
                    writer.writeln_fmt(
                        format_args!("node := pParent.AddChild('{}');", variable.xml_name,),
                        Some(2),
                    )?;
                    writer.writeln_fmt(
                        format_args!(
                            "{}.AppendToXmlRaw(node);",
                            Helper::as_variable_name(&variable.name),
                        ),
                        Some(2),
                    )?;
                }
                DataType::List(lt) => {
                    writer.writeln_fmt(
                        format_args!(
                            "for var {} in {} do begin",
                            variable.name,
                            Helper::as_variable_name(&variable.name),
                        ),
                        Some(2),
                    )?;
                    Self::generate_list_to_xml(
                        writer,
                        lt,
                        &Helper::as_variable_name(&variable.name),
                        &variable.xml_name,
                        type_aliases,
                        4,
                    )?;
                    writer.writeln("end;", Some(2))?;
                }
                DataType::FixedSizeList(item_type, size) => {
                    for i in 1..size + 1 {
                        Self::generate_list_to_xml(
                            writer,
                            item_type,
                            &(Helper::first_char_uppercase(&variable.name)
                                + i.to_string().as_str()),
                            &variable.xml_name,
                            type_aliases,
                            2,
                        )?;

                        if i < *size {
                            writer.newline()?;
                        }
                    }
                }
                _ => {
                    for arg in Self::generate_standard_type_to_xml(
                        &variable.data_type,
                        &Helper::as_variable_name(&variable.name),
                        &variable.xml_name,
                        None,
                    ) {
                        writer.writeln(arg.as_str(), Some(2))?;
                    }
                }
            }

            if index < class_type.variables.len() - 1 {
                writer.newline()?;
            }
        }
        writer.writeln("end;", None)?;
        Ok(())
    }

    fn generate_list_to_xml<T: Write>(
        writer: &mut CodeWriter<T>,
        data_type: &DataType,
        variable_name: &String,
        xml_name: &String,
        type_aliases: &[TypeAlias],
        indentation: usize,
    ) -> Result<(), CodeGenError> {
        match data_type {
            DataType::Enumeration(_) => {
                writer.writeln_fmt(
                    format_args!("node := pParent.AddChild('{}');", xml_name),
                    Some(indentation),
                )?;

                writer.writeln_fmt(
                    format_args!("node.Text := {}.ToXmlValue;", variable_name,),
                    Some(indentation),
                )?;
            }
            DataType::Alias(name) => {
                if let Some((data_type, pattern)) =
                    Helper::get_alias_data_type(name.as_str(), type_aliases)
                {
                    for arg in Self::generate_standard_type_to_xml(
                        &data_type,
                        variable_name,
                        xml_name,
                        pattern,
                    ) {
                        writer.writeln(arg.as_str(), Some(indentation))?;
                    }
                }
            }
            DataType::Custom(_) => {
                writer.writeln_fmt(
                    format_args!("node := pParent.AddChild('{}');", xml_name),
                    Some(indentation),
                )?;
                writer.writeln_fmt(
                    format_args!("{}.AppendToXmlRaw(node);", variable_name,),
                    Some(indentation),
                )?;
            }
            DataType::List(_) => (),
            _ => {
                for arg in
                    Self::generate_standard_type_to_xml(data_type, variable_name, xml_name, None)
                {
                    writer.writeln(arg.as_str(), Some(indentation))?;
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
                "{} := ({}.Text = cnXmlTrueValue) or ({}.Text = '1');",
                variable_name, node, node
            ),
            DataType::DateTime | DataType::Date if pattern.is_some() => format!(
                "{} := DecodeDateTime({}.Text, '{}');",
                variable_name,
                node,
                pattern.unwrap_or_default(),
            ),
            DataType::DateTime | DataType::Date => {
                format!("{} := ISO8601ToDate({}.Text);", variable_name, node,)
            }
            DataType::Double => format!("{} := StrToFloat({}.Text);", variable_name, node),
            DataType::Binary(BinaryEncoding::Base64) => format!(
                "{} := TNetEncoding.Base64.DecodeStringToBytes({}.Text);",
                variable_name, node
            ),
            DataType::Binary(BinaryEncoding::Hex) => format!(
                "HexToBin({}.Text, 0, {}, 0, Length({}.Text));",
                node, variable_name, node,
            ),
            DataType::String => format!("{} := {}.Text;", variable_name, node),
            DataType::Time if pattern.is_some() => format!(
                "{} := TimeOf(DecodeDateTime({}.Text, '{}'));",
                variable_name,
                node,
                pattern.unwrap_or_default(),
            ),
            DataType::Time => format!("{} := TimeOf(ISO8601ToDate({}.Text));", variable_name, node),
            DataType::SmallInteger
            | DataType::ShortInteger
            | DataType::Integer
            | DataType::LongInteger
            | DataType::UnsignedSmallInteger
            | DataType::UnsignedShortInteger
            | DataType::UnsignedInteger
            | DataType::UnsignedLongInteger => {
                format!("{} := StrToInt({}.Text);", variable_name, node)
            }
            _ => String::new(),
        }
    }

    fn generate_standard_type_to_xml(
        data_type: &DataType,
        variable_name: &String,
        xml_name: &String,
        pattern: Option<String>,
    ) -> Vec<String> {
        match data_type {
            DataType::Alias(_)
            | DataType::Custom(_)
            | DataType::Enumeration(_)
            | DataType::List(_)
            | DataType::FixedSizeList(_, _)
            | DataType::Union(_) => vec![],
            _ => vec![
                format!("node := pParent.AddChild('{}');", xml_name),
                format!(
                    "node.Text := {};",
                    Helper::get_variable_value_as_string(data_type, variable_name, pattern),
                ),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    // use pretty_assertions::assert_eq;
    
    // use super::*;

    // TODO: Write Test
}
