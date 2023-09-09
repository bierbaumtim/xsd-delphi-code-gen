use std::io::Write;

use crate::generator::{
    code_generator_trait::{CodeGenError, CodeGenOptions},
    internal_representation::DOCUMENT_NAME,
    types::{BinaryEncoding, ClassType, DataType, TypeAlias, Variable},
};

use super::{
    code_writer::{CodeWriter, FunctionModifier, FunctionType},
    helper::Helper,
};

impl DataType {
    fn is_reference_type(&self, type_aliases: &[TypeAlias]) -> bool {
        match self {
            DataType::Alias(n) => Helper::get_alias_data_type(n.as_str(), type_aliases)
                .map_or(true, |(dt, _)| dt.is_reference_type(type_aliases)),
            DataType::Custom(_) | DataType::List(_) | DataType::InlineList(_) => true,
            DataType::FixedSizeList(dt, _) => dt.as_ref().is_reference_type(type_aliases),
            _ => false,
        }
    }
}

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
        type_aliases: &[TypeAlias],
        options: &CodeGenOptions,
        indentation: usize,
    ) -> Result<(), CodeGenError> {
        writer.writeln("{$REGION 'Declarations}", Some(2))?;

        Self::generate_class_declaration(writer, document, type_aliases, options, indentation)?;

        for class_type in classes {
            if class_type.name == DOCUMENT_NAME {
                continue;
            }

            Self::generate_class_declaration(
                writer,
                class_type,
                type_aliases,
                options,
                indentation,
            )?;
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
        type_aliases: &[TypeAlias],
        options: &CodeGenOptions,
        indentation: usize,
    ) -> Result<(), CodeGenError> {
        writer.write_documentation(&class_type.documentations, Some(indentation))?;
        Helper::write_qualified_name_comment(
            writer,
            &class_type.qualified_name,
            Some(indentation),
        )?;

        writer.writeln_fmt(
            format_args!(
                "{} = class{}",
                Helper::as_type_name(&class_type.name, &options.type_prefix),
                class_type.super_type.as_ref().map_or_else(
                    || "(TObject)".to_owned(),
                    |(n, _)| format!("({})", Helper::as_type_name(n, &options.type_prefix))
                )
            ),
            Some(indentation),
        )?;

        let optional_variable_count = class_type
            .variables
            .iter()
            .filter(|v| !v.required && !v.data_type.is_reference_type(type_aliases))
            .count();

        if optional_variable_count > 0 {
            writer.writeln("strict private", Some(indentation))?;

            Self::generate_optional_properties_backing_fields_declaration(
                writer,
                class_type,
                type_aliases,
                options,
                optional_variable_count,
                indentation,
            )?;
        }

        writer.writeln("public", Some(indentation))?;

        // Variables
        for variable in class_type
            .variables
            .iter()
            .filter(|v| v.required || v.data_type.is_reference_type(type_aliases))
        {
            let variable_name = Helper::as_variable_name(&variable.name);

            match &variable.data_type {
                DataType::Alias(n) => {
                    if let Some((data_type, _)) =
                        Helper::get_alias_data_type(n.as_str(), type_aliases)
                    {
                        if variable.required {
                            Helper::write_required_comment(writer, Some(indentation + 2))?;
                        }

                        let rhs = if let DataType::InlineList(_) = data_type {
                            Helper::as_type_name(n, &options.type_prefix)
                        } else {
                            Helper::get_datatype_language_representation(
                                &variable.data_type,
                                &options.type_prefix,
                            )
                        };

                        writer.writeln_fmt(
                            format_args!("{variable_name}: {rhs};"),
                            Some(indentation + 2),
                        )?;
                    } else {
                        return Err(CodeGenError::MissingDataType(
                            class_type.name.clone(),
                            variable_name,
                        ));
                    }
                }
                DataType::FixedSizeList(item_type, size) => {
                    let lang_rep = Helper::get_datatype_language_representation(
                        item_type,
                        &options.type_prefix,
                    );

                    for i in 1..=*size {
                        if variable.required {
                            Helper::write_required_comment(writer, Some(indentation + 2))?;
                        }

                        writer.writeln_fmt(
                            format_args!("{variable_name}{i}: {lang_rep};"),
                            Some(indentation + 2),
                        )?;
                    }
                }
                _ => {
                    let lang_rep = Helper::get_datatype_language_representation(
                        &variable.data_type,
                        &options.type_prefix,
                    );

                    if variable.required {
                        Helper::write_required_comment(writer, Some(indentation + 2))?;
                    }

                    writer.writeln_fmt(
                        format_args!("{variable_name}: {lang_rep};"),
                        Some(indentation + 2),
                    )?;
                }
            }
        }

        if optional_variable_count < class_type.variables.len() {
            writer.newline()?;
        }

        let fn_decorator = class_type
            .super_type
            .as_ref()
            .map_or(FunctionModifier::Virtual, |_| FunctionModifier::Override);

        // constructors and destructors
        if options.generate_to_xml {
            writer.write_default_constructor(Some(fn_decorator.clone()), Some(indentation + 2))?;
        }
        if options.generate_from_xml {
            writer.write_constructor(
                "FromXml",
                Some(vec![("node", "IXMLNode")]),
                Some(fn_decorator.clone()),
                Some(indentation + 2),
            )?;
        }

        if class_type
            .variables
            .iter()
            .any(|v| v.requires_free || !v.required)
        {
            writer.write_destructor(Some(indentation + 2))?;
        }

        if options.generate_to_xml {
            writer.newline()?;
            writer.write_function_declaration(
                FunctionType::Procedure,
                "AppendToXmlRaw",
                Some(vec![("pParent", "IXMLNode")]),
                false,
                Some(vec![fn_decorator.clone()]),
                indentation + 2,
            )?;

            if class_type.name == DOCUMENT_NAME {
                writer.newline()?;
                writer.write_function_declaration(
                    FunctionType::Function(String::from("String")),
                    "ToXml",
                    None,
                    false,
                    Some(vec![fn_decorator]),
                    indentation + 2,
                )?;
            }
        }

        // Properties for optional value
        if optional_variable_count > 0 {
            Self::generate_optional_properties_declaration(
                writer,
                class_type,
                type_aliases,
                options,
                indentation,
            )?;
        }

        writer.writeln("end;", Some(indentation))?;
        writer.newline()?;

        Ok(())
    }

    fn generate_optional_properties_backing_fields_declaration<T: Write>(
        writer: &mut CodeWriter<T>,
        class_type: &ClassType,
        type_aliases: &[TypeAlias],
        options: &CodeGenOptions,
        optional_variable_count: usize,
        indentation: usize,
    ) -> Result<(), CodeGenError> {
        let mut setter = Vec::with_capacity(optional_variable_count);

        for variable in class_type
            .variables
            .iter()
            .filter(|v| !v.required && !v.data_type.is_reference_type(type_aliases))
        {
            let variable_name = Helper::as_variable_name(&variable.name);

            if let DataType::FixedSizeList(item_type, size) = &variable.data_type {
                let lang_rep =
                    Helper::get_datatype_language_representation(item_type, &options.type_prefix);

                for i in 1..=*size {
                    writer.writeln_fmt(
                        format_args!("F{variable_name}{i}: {lang_rep};"),
                        Some(indentation + 2),
                    )?;

                    setter.push(format!(
                        "procedure Set{variable_name}{i}(pValue: TOptional<{lang_rep}>);"
                    ));
                }
            } else {
                let lang_rep = Helper::get_datatype_language_representation(
                    &variable.data_type,
                    &options.type_prefix,
                );

                writer.writeln_fmt(
                    format_args!("F{variable_name}: TOptional<{lang_rep}>;"),
                    Some(indentation + 2),
                )?;

                setter.push(format!(
                    "procedure Set{variable_name}(pValue: TOptional<{lang_rep}>);"
                ));
            }
        }
        writer.newline()?;
        for line in setter {
            writer.writeln(line.as_str(), Some(indentation + 2))?;
        }

        Ok(())
    }

    fn generate_optional_properties_declaration<T: Write>(
        writer: &mut CodeWriter<T>,
        class_type: &ClassType,
        type_aliases: &[TypeAlias],
        options: &CodeGenOptions,
        indentation: usize,
    ) -> Result<(), CodeGenError> {
        writer.newline()?;
        for variable in class_type
            .variables
            .iter()
            .filter(|v| !v.required && !v.data_type.is_reference_type(type_aliases))
        {
            let variable_name = Helper::as_variable_name(&variable.name);

            if let DataType::FixedSizeList(item_type, size) = &variable.data_type {
                let lang_rep =
                    Helper::get_datatype_language_representation(item_type, &options.type_prefix);

                for i in 1..=*size {
                    writer.writeln_fmt(
                        format_args!("F{variable_name}{i}: {lang_rep};"),
                        Some(indentation + 2),
                    )?;

                    writer.writeln_fmt(
                        format_args!(
                            "property {variable_name}{i}: TOptional<{lang_rep}> read F{variable_name}{i} write Set{variable_name}{i};"
                        ),
                        Some(indentation + 2),
                    )?;
                }
            } else {
                let lang_rep = Helper::get_datatype_language_representation(
                    &variable.data_type,
                    &options.type_prefix,
                );

                writer.writeln_fmt(
                    format_args!(
                        "property {variable_name}: TOptional<{lang_rep}> read F{variable_name} write Set{variable_name};"
                    ),
                    Some(indentation + 2),
                )?;
            }
        }

        Ok(())
    }

    fn generate_class_implementation<T: Write>(
        writer: &mut CodeWriter<T>,
        class_type: &ClassType,
        type_aliases: &[TypeAlias],
        options: &CodeGenOptions,
    ) -> Result<(), CodeGenError> {
        let formated_name = Helper::as_type_name(&class_type.name, &options.type_prefix);
        let needs_destroy = class_type
            .variables
            .iter()
            .any(|v| v.requires_free || !v.required);

        writer.writeln_fmt(format_args!("{{ {formated_name} }}"), None)?;

        if options.generate_to_xml {
            Self::generate_constructor_implementation(
                writer,
                &formated_name,
                class_type,
                type_aliases,
                options,
            )?;
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

        Self::generate_optional_properties_setter(writer, class_type, type_aliases, options)?;

        if needs_destroy {
            writer.newline()?;
            writer.writeln_fmt(format_args!("destructor {formated_name}.Destroy;"), None)?;

            writer.writeln("begin", None)?;

            for variable in class_type
                .variables
                .iter()
                .filter(|v| v.requires_free || !v.required)
            {
                match &variable.data_type {
                    DataType::FixedSizeList(_, size) => {
                        for i in 1..=*size {
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
        type_aliases: &[TypeAlias],
        options: &CodeGenOptions,
    ) -> Result<(), CodeGenError> {
        writer.writeln_fmt(format_args!("constructor {formated_name}.Create;"), None)?;
        writer.writeln("begin", None)?;

        if class_type.super_type.is_some() {
            writer.writeln("inherited;", Some(2))?;
            writer.newline()?;
        }

        for variable in &class_type.variables {
            let variable_name = Helper::as_variable_name(&variable.name);

            match &variable.data_type {
                DataType::Alias(name) => {
                    if let Some((data_type, _)) =
                        Helper::get_alias_data_type(name.as_str(), type_aliases)
                    {
                        match data_type {
                            DataType::InlineList(_) => {
                                writer.write_variable_initialization(
                                    variable_name.as_str(),
                                    Helper::get_datatype_language_representation(
                                        &variable.data_type,
                                        &options.type_prefix,
                                    )
                                    .as_str(),
                                    variable.required,
                                    false,
                                    Some(2),
                                )?;
                            }
                            _ => {
                                writer.write_variable_initialization(
                                    variable_name.as_str(),
                                    Helper::as_type_name(name, &options.type_prefix).as_str(),
                                    variable.required,
                                    true,
                                    Some(2),
                                )?;
                            }
                        }
                    }
                }
                DataType::Enumeration(name) => {
                    writer.write_variable_initialization(
                        variable_name.as_str(),
                        Helper::as_type_name(name, &options.type_prefix).as_str(),
                        variable.required,
                        true,
                        Some(2),
                    )?;
                }
                DataType::Custom(name) => {
                    writer.write_variable_initialization(
                        variable_name.as_str(),
                        Helper::as_type_name(name, &options.type_prefix).as_str(),
                        variable.required,
                        false,
                        Some(2),
                    )?;
                }
                DataType::List(_) | DataType::InlineList(_) => {
                    writer.write_variable_initialization(
                        variable_name.as_str(),
                        Helper::get_datatype_language_representation(
                            &variable.data_type,
                            &options.type_prefix,
                        )
                        .as_str(),
                        true,
                        false,
                        Some(2),
                    )?;
                }
                DataType::FixedSizeList(item_type, size) => {
                    let rhs = match item_type.as_ref() {
                        DataType::Alias(name) => {
                            if let Some((data_type, _)) =
                                Helper::get_alias_data_type(name.as_str(), type_aliases)
                            {
                                let type_name = Helper::as_type_name(name, &options.type_prefix);

                                match data_type {
                                    DataType::Custom(_) => String::from("nil"),
                                    _ if variable.required => format!("Default({type_name})"),
                                    _ => format!("TNone<{type_name}>.Create"),
                                }
                            } else {
                                return Err(CodeGenError::MissingDataType(
                                    class_type.name.clone(),
                                    variable_name,
                                ));
                            }
                        }
                        DataType::Enumeration(name) => {
                            let type_name = Helper::as_type_name(name, &options.type_prefix);

                            if variable.required {
                                format!("Default({type_name})")
                            } else {
                                format!("TNone<{type_name}>.Create")
                            }
                        }
                        DataType::Custom(name) => {
                            if variable.required {
                                format!(
                                    "{}.Create",
                                    Helper::as_type_name(name, &options.type_prefix)
                                )
                            } else {
                                String::from("nil")
                            }
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
                        _ => {
                            let lang_rep = Helper::get_datatype_language_representation(
                                item_type.as_ref(),
                                &options.type_prefix,
                            );

                            if variable.required {
                                format!("Default({lang_rep})")
                            } else {
                                format!("TNone<{lang_rep}>.Create")
                            }
                        }
                    };

                    for i in 1..=*size {
                        writer
                            .writeln_fmt(format_args!("{variable_name}{i} := {rhs};"), Some(2))?;
                    }
                }
                DataType::Uri => {
                    if variable.required {
                        writer.writeln_fmt(
                            format_args!("{variable_name} := TURI.Create('');"),
                            Some(2),
                        )?;
                    } else {
                        writer.writeln_fmt(
                            format_args!("{variable_name} := TNone<TURI>.Create;"),
                            Some(2),
                        )?;
                    }
                }
                _ => {
                    writer.write_variable_initialization(
                        variable_name.as_str(),
                        Helper::get_datatype_language_representation(
                            &variable.data_type,
                            &options.type_prefix,
                        )
                        .as_str(),
                        variable.required,
                        true,
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
            format_args!("constructor {formated_name}.FromXml(node: IXMLNode);"),
            None,
        )?;
        writer.writeln("begin", None)?;

        if class_type.super_type.is_some() {
            writer.writeln("inherited;", Some(2))?;
            writer.newline()?;
        }

        if class_type.variables.iter().any(|v| !v.required) {
            writer.writeln("var vOptionalNode: IXMLNode;", Some(2))?;
            writer.newline()?;
        }

        for variable in &class_type.variables {
            let variable_name = Helper::as_variable_name(&variable.name);

            match &variable.data_type {
                DataType::Alias(name) => {
                    if let Some((data_type, pattern)) =
                        Helper::get_alias_data_type(name.as_str(), type_aliases)
                    {
                        match data_type {
                            DataType::InlineList(lt) => {
                                Self::generate_inline_list_from_xml(
                                    writer,
                                    type_aliases,
                                    options,
                                    variable,
                                    lt.as_ref(),
                                )?;
                            }
                            _ => writer.writeln_fmt(
                                format_args!(
                                    "{} := {};",
                                    variable_name,
                                    Self::generate_standard_type_from_xml(
                                        &data_type,
                                        format!("node.ChildNodes['{}'].Text", variable.xml_name),
                                        pattern,
                                    )
                                ),
                                Some(2),
                            )?,
                        }
                    }
                }
                DataType::Enumeration(name) => {
                    let type_name = Helper::as_type_name(name, &options.type_prefix);

                    if variable.required {
                        writer.writeln_fmt(
                            format_args!(
                                "{} := {}.FromXmlValue(node.ChildNodes['{}'].Text);",
                                variable_name, type_name, variable.xml_name
                            ),
                            Some(2),
                        )?;
                    } else {
                        writer.writeln_fmt(
                            format_args!(
                                "vOptionalNode := node.ChildNodes.FindNode('{}');",
                                variable.xml_name
                            ),
                            Some(2),
                        )?;
                        writer.writeln("if Assigned(vOptionalNode) then begin", Some(2))?;
                        writer.writeln_fmt(
                            format_args!(
                                "{variable_name} := TSome<{type_name}>.Create({type_name}.FromXmlValue(vOptionalNode.Text));"
                            ),
                            Some(4),
                        )?;
                        writer.writeln("end else begin", Some(2))?;
                        writer.writeln_fmt(
                            format_args!("{variable_name} := TNone<{type_name}>.Create"),
                            Some(4),
                        )?;
                        writer.writeln("end;", Some(2))?;
                        writer.newline()?;
                    }
                }
                DataType::Custom(name) => {
                    let type_name = Helper::as_type_name(name, &options.type_prefix);

                    if variable.required {
                        writer.writeln_fmt(
                            format_args!(
                                "{} := {}.FromXml(node.ChildNodes['{}']);",
                                variable_name, type_name, variable.xml_name
                            ),
                            Some(2),
                        )?;
                    } else {
                        writer.writeln_fmt(
                            format_args!(
                                "vOptionalNode := node.ChildNodes.FindNode('{}');",
                                variable.xml_name
                            ),
                            Some(2),
                        )?;
                        writer.writeln("if Assigned(vOptionalNode) then begin", Some(2))?;
                        writer.writeln_fmt(
                            format_args!("{variable_name} := {type_name}.FromXml(vOptionalNode);"),
                            Some(4),
                        )?;
                        writer.writeln("end else begin", Some(2))?;
                        writer.writeln_fmt(format_args!("{variable_name} := nil;"), Some(4))?;
                        writer.writeln("end;", Some(2))?;
                        writer.newline()?;
                    }
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
                        *size,
                    )?;
                }
                DataType::InlineList(lt) => {
                    Self::generate_inline_list_from_xml(
                        writer,
                        type_aliases,
                        options,
                        variable,
                        lt.as_ref(),
                    )?;
                }
                _ => {
                    if variable.required {
                        writer.writeln_fmt(
                            format_args!(
                                "{} := {};",
                                variable_name,
                                Self::generate_standard_type_from_xml(
                                    &variable.data_type,
                                    format!("node.ChildNodes['{}'].Text", variable.xml_name),
                                    None,
                                )
                            ),
                            Some(2),
                        )?;
                    } else {
                        let lang_rep = Helper::get_datatype_language_representation(
                            &variable.data_type,
                            &options.type_prefix,
                        );

                        writer.writeln_fmt(
                            format_args!(
                                "vOptionalNode := node.ChildNodes.FindNode('{}');",
                                variable.xml_name
                            ),
                            Some(2),
                        )?;
                        writer.writeln("if Assigned(vOptionalNode) then begin", Some(2))?;
                        writer.writeln_fmt(
                            format_args!(
                                "{} := TSome<{}>.Create({});",
                                variable_name,
                                lang_rep,
                                Self::generate_standard_type_from_xml(
                                    &variable.data_type,
                                    "vOptionalNode.Text".to_owned(),
                                    None,
                                ),
                            ),
                            Some(4),
                        )?;
                        writer.writeln("end else begin", Some(2))?;
                        writer.writeln_fmt(
                            format_args!("{variable_name} := TNone<{lang_rep}>.Create"),
                            Some(4),
                        )?;
                        writer.writeln("end;", Some(2))?;
                        writer.newline()?;
                    }
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
                            "var {} := {};",
                            Helper::as_variable_name(&variable.name),
                            Self::generate_standard_type_from_xml(
                                &data_type,
                                format!("__{}Node.Text", variable.name),
                                pattern,
                            )
                        ),
                        Some(6),
                    )?;
                    writer.writeln_fmt(
                        format_args!("{formatted_variable_name}.Add(__{formatted_variable_name});"),
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
                        "var __{} := {};",
                        formatted_variable_name,
                        Self::generate_standard_type_from_xml(
                            item_type,
                            format!("__{}Node.Text", variable.name),
                            None,
                        )
                    ),
                    Some(6),
                )?;
                writer.writeln_fmt(
                    format_args!("{formatted_variable_name}.Add(__{formatted_variable_name});"),
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
        size: usize,
    ) -> Result<(), CodeGenError> {
        for i in 1..=size {
            writer.writeln_fmt(
                format_args!(
                    "{}{} := Default({});",
                    Helper::as_variable_name(&variable.name),
                    i,
                    Helper::get_datatype_language_representation(item_type, &options.type_prefix),
                ),
                Some(2),
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
        for i in 1..=size {
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
                                "{}: {}{} := {};",
                                i - 1,
                                Helper::as_variable_name(&variable.name),
                                i,
                                Self::generate_standard_type_from_xml(
                                    &data_type,
                                    format!("__{}Node.Text", variable.name),
                                    pattern,
                                )
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
                            "{}: {}{} := {};",
                            i - 1,
                            Helper::as_variable_name(&variable.name),
                            i,
                            Self::generate_standard_type_from_xml(
                                item_type,
                                format!("__{}Node.Text", variable.name),
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

    fn generate_inline_list_from_xml<T: Write>(
        writer: &mut CodeWriter<T>,
        type_aliases: &[TypeAlias],
        options: &CodeGenOptions,
        variable: &Variable,
        item_type: &DataType,
    ) -> Result<(), CodeGenError> {
        let formatted_variable_name = Helper::as_variable_name(&variable.name);
        let mut indentation = 2;

        writer.newline()?;
        writer.writeln_fmt(
            format_args!(
                "{} := {}.Create;",
                formatted_variable_name,
                Helper::get_datatype_language_representation(
                    &variable.data_type,
                    &options.type_prefix
                ),
            ),
            Some(indentation),
        )?;

        if variable.required {
            writer.writeln_fmt(
                format_args!(
                    "for var vPart in node.ChildNodes['{}'].Text.Split([' ']) do begin",
                    variable.xml_name,
                ),
                Some(indentation),
            )?;
        } else {
            writer.writeln_fmt(
                format_args!(
                    "vOptionalNode := node.ChildNodes.FindNode('{}');",
                    variable.xml_name
                ),
                Some(indentation),
            )?;
            writer.writeln("if Assigned(vOptionalNode) then begin", Some(indentation))?;

            indentation = 4;
            writer.writeln(
                "for var vPart in vOptionalNode.Text.Split([' ']) do begin",
                Some(indentation),
            )?;
        }

        match item_type {
            DataType::Alias(name) => {
                if let Some((data_type, pattern)) =
                    Helper::get_alias_data_type(name.as_str(), type_aliases)
                {
                    writer.writeln_fmt(
                        format_args!(
                            "{}.Add({});",
                            formatted_variable_name,
                            Self::generate_standard_type_from_xml(
                                &data_type,
                                "vPart".to_owned(),
                                pattern,
                            ),
                        ),
                        Some(indentation + 2),
                    )?;
                }
            }
            DataType::Enumeration(n) | DataType::Union(n) => {
                writer.writeln_fmt(
                    format_args!(
                        "{}.Add({}Helper.FromXmlValue(vPart));",
                        formatted_variable_name,
                        Helper::as_type_name(n, &options.type_prefix),
                    ),
                    Some(indentation + 2),
                )?;
            }
            DataType::Custom(_)
            | DataType::List(_)
            | DataType::FixedSizeList(_, _)
            | DataType::InlineList(_) => todo!(),
            _ => writer.writeln_fmt(
                format_args!(
                    "{}.Add({});",
                    formatted_variable_name,
                    Self::generate_standard_type_from_xml(item_type, "vPart".to_owned(), None),
                ),
                Some(indentation + 2),
            )?,
        };

        if !variable.required {
            writer.writeln("end;", Some(indentation))?;
        }

        writer.writeln("end;", Some(2))?;

        Ok(())
    }

    fn generate_document_to_xml_implementation<T: Write>(
        writer: &mut CodeWriter<T>,
        formated_name: &String,
    ) -> Result<(), std::io::Error> {
        writer.writeln_fmt(
            format_args!("function {formated_name}.ToXml: String;"),
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
            format_args!("procedure {formated_name}.AppendToXmlRaw(pParent: IXMLNode);"),
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
            let variable_name = Helper::as_variable_name(&variable.name);
            let mut indentation = 2;

            match &variable.data_type {
                DataType::Alias(name) => {
                    if let Some((data_type, pattern)) =
                        Helper::get_alias_data_type(name.as_str(), type_aliases)
                    {
                        match data_type {
                            DataType::InlineList(lt) => {
                                if variable.required {
                                    writer.writeln_fmt(
                                        format_args!(
                                            "if Assigned({variable_name}) and {variable_name}.Count > 0 then begin"
                                        ),
                                        Some(2),
                                    )?;
                                    indentation = 4;
                                }

                                writer.writeln_fmt(
                                    format_args!(
                                        "node := pParent.AddChild('{}');",
                                        variable.xml_name
                                    ),
                                    Some(indentation),
                                )?;
                                writer.writeln_fmt(
                                    format_args!(
                                        "for var I := 0 to {variable_name}.Count - 1 do begin"
                                    ),
                                    Some(indentation),
                                )?;
                                writer.writeln_fmt(
                                    format_args!(
                                        "node.Text := node.Text + {};",
                                        Helper::get_variable_value_as_string(
                                            lt.as_ref(),
                                            &format!("{variable_name}[I]"),
                                            &pattern
                                        )
                                    ),
                                    Some(indentation + 2),
                                )?;
                                writer.newline()?;
                                writer.writeln_fmt(
                                    format_args!("if I < {variable_name}.Count - 1 then begin"),
                                    Some(4),
                                )?;
                                writer.writeln(
                                    "node.Text := node.Text + ' ';",
                                    Some(indentation + 4),
                                )?;
                                writer.writeln("end;", Some(indentation + 2))?;
                                writer.writeln("end;", Some(indentation))?;

                                if variable.required {
                                    writer.writeln("end;", Some(2))?;
                                }
                            }
                            _ => {
                                if variable.required {
                                    for arg in Self::generate_standard_type_to_xml(
                                        &data_type,
                                        &variable_name,
                                        &variable.xml_name,
                                        &pattern,
                                    ) {
                                        writer.writeln(arg.as_str(), Some(indentation))?;
                                    }
                                } else {
                                    writer.writeln_fmt(
                                        format_args!("if {variable_name}.IsSome then begin"),
                                        Some(2),
                                    )?;
                                    for arg in Self::generate_standard_type_to_xml(
                                        &data_type,
                                        &(variable_name.clone() + ".Unwrap"),
                                        &variable.xml_name,
                                        &pattern,
                                    ) {
                                        writer.writeln(arg.as_str(), Some(indentation + 2))?;
                                    }
                                    writer.writeln("end;", Some(2))?;
                                }
                            }
                        }
                    }
                }
                DataType::Enumeration(_) => {
                    if variable.required {
                        writer.writeln_fmt(
                            format_args!("node := pParent.AddChild('{}');", variable.xml_name),
                            Some(indentation),
                        )?;

                        writer.writeln_fmt(
                            format_args!("node.Text := {variable_name}.ToXmlValue;"),
                            Some(indentation),
                        )?;
                    } else {
                        writer.writeln_fmt(
                            format_args!("if {variable_name}.IsSome then begin"),
                            Some(2),
                        )?;
                        writer.writeln_fmt(
                            format_args!("node := pParent.AddChild('{}');", variable.xml_name),
                            Some(indentation + 2),
                        )?;

                        writer.writeln_fmt(
                            format_args!("node.Text := {variable_name}.Unwrap.ToXmlValue;"),
                            Some(indentation + 2),
                        )?;
                        writer.writeln("end;", Some(2))?;
                    }
                }
                DataType::Custom(_) => {
                    if !variable.required {
                        writer.writeln_fmt(
                            format_args!("if Assigned({variable_name}) then begin"),
                            Some(2),
                        )?;

                        indentation = 4;
                    }
                    writer.writeln_fmt(
                        format_args!("node := pParent.AddChild('{}');", variable.xml_name),
                        Some(indentation),
                    )?;
                    writer.writeln_fmt(
                        format_args!("{variable_name}.AppendToXmlRaw(node);"),
                        Some(indentation),
                    )?;
                    if !variable.required {
                        writer.writeln("end;", Some(2))?;
                    }
                }
                DataType::List(lt) => {
                    writer.writeln_fmt(
                        format_args!("for var {} in {} do begin", variable.name, variable_name),
                        Some(indentation),
                    )?;
                    Self::generate_list_to_xml(
                        writer,
                        lt,
                        &variable_name,
                        &variable.xml_name,
                        type_aliases,
                        indentation + 2,
                    )?;
                    writer.writeln("end;", Some(indentation))?;
                }
                DataType::FixedSizeList(item_type, size) => {
                    for i in 1..=*size {
                        // TODO: Abhängig vom DataType Assigned oder Unwrap
                        Self::generate_list_to_xml(
                            writer,
                            item_type,
                            &(Helper::first_char_uppercase(&variable.name)
                                + i.to_string().as_str()),
                            &variable.xml_name,
                            type_aliases,
                            indentation,
                        )?;

                        if i < *size {
                            writer.newline()?;
                        }
                    }
                }
                _ => {
                    if variable.required {
                        for arg in Self::generate_standard_type_to_xml(
                            &variable.data_type,
                            &variable_name,
                            &variable.xml_name,
                            &None,
                        ) {
                            writer.writeln(arg.as_str(), Some(indentation))?;
                        }
                    } else {
                        writer.writeln_fmt(
                            format_args!("if {variable_name}.IsSome then begin"),
                            Some(2),
                        )?;
                        for arg in Self::generate_standard_type_to_xml(
                            &variable.data_type,
                            &(variable_name.clone() + ".Unwrap"),
                            &variable.xml_name,
                            &None,
                        ) {
                            writer.writeln(arg.as_str(), Some(indentation + 2))?;
                        }
                        writer.writeln("end;", Some(2))?;
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
                    format_args!("node := pParent.AddChild('{xml_name}');"),
                    Some(indentation),
                )?;

                writer.writeln_fmt(
                    format_args!("node.Text := {variable_name}.ToXmlValue;"),
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
                        &pattern,
                    ) {
                        writer.writeln(arg.as_str(), Some(indentation))?;
                    }
                }
            }
            DataType::Custom(_) => {
                writer.writeln_fmt(
                    format_args!("node := pParent.AddChild('{xml_name}');"),
                    Some(indentation),
                )?;
                writer.writeln_fmt(
                    format_args!("{variable_name}.AppendToXmlRaw(node);"),
                    Some(indentation),
                )?;
            }
            DataType::List(_) => (),
            _ => {
                for arg in
                    Self::generate_standard_type_to_xml(data_type, variable_name, xml_name, &None)
                {
                    writer.writeln(arg.as_str(), Some(indentation))?;
                }
            }
        }

        Ok(())
    }

    fn generate_optional_properties_setter<T: Write>(
        writer: &mut CodeWriter<T>,
        class_type: &ClassType,
        type_aliases: &[TypeAlias],
        options: &CodeGenOptions,
    ) -> Result<(), std::io::Error> {
        let optional_variables_count = class_type
            .variables
            .iter()
            .filter(|v| !v.required && !v.data_type.is_reference_type(type_aliases))
            .count();

        if optional_variables_count == 0 {
            return Ok(());
        }

        writer.newline()?;

        let class_type_name = Helper::as_type_name(&class_type.name, &options.type_prefix);

        for (i, variable) in class_type
            .variables
            .iter()
            .filter(|v| !v.required && !v.data_type.is_reference_type(type_aliases))
            .enumerate()
        {
            let variable_name = Helper::as_variable_name(&variable.name);

            if let DataType::FixedSizeList(item_type, size) = &variable.data_type {
                let lang_rep =
                    Helper::get_datatype_language_representation(item_type, &options.type_prefix);

                for i in 1..=*size {
                    writer.writeln_fmt(
                        format_args!(
                            "procedure {class_type_name}.Set{variable_name}{i}(pValue: TOptional<{lang_rep}>);"
                        ),
                        None,
                    )?;
                    writer.writeln("begin", None)?;
                    writer.writeln_fmt(
                        format_args!(
                            "if F{variable_name}{i} <> pValue then F{variable_name}.Free;"
                        ),
                        Some(2),
                    )?;
                    writer.newline()?;
                    writer.writeln_fmt(format_args!("F{variable_name}{i} := pValue;"), Some(2))?;
                    writer.writeln("end;", None)?;

                    if i < *size {
                        writer.newline()?;
                    }
                }
            } else {
                let lang_rep = Helper::get_datatype_language_representation(
                    &variable.data_type,
                    &options.type_prefix,
                );

                writer.writeln_fmt(
                    format_args!(
                        "procedure {class_type_name}.Set{variable_name}(pValue: TOptional<{lang_rep}>);"
                    ),
                    None,
                )?;
                writer.writeln("begin", None)?;
                writer.writeln_fmt(
                    format_args!("if F{variable_name} <> pValue then F{variable_name}.Free;"),
                    Some(2),
                )?;
                writer.newline()?;
                writer.writeln_fmt(format_args!("F{variable_name} := pValue;"), Some(2))?;
                writer.writeln("end;", None)?;
            }

            if i < optional_variables_count - 1 {
                writer.newline()?;
            }
        }

        Ok(())
    }

    fn generate_standard_type_from_xml(
        data_type: &DataType,
        value: String,
        pattern: Option<String>,
    ) -> String {
        match data_type {
            DataType::Boolean => format!("({value} = cnXmlTrueValue) or ({value} = '1')"),
            DataType::DateTime | DataType::Date if pattern.is_some() => format!(
                "DecodeDateTime({}, '{}')",
                value,
                pattern.unwrap_or_default(),
            ),
            DataType::DateTime | DataType::Date => format!("ISO8601ToDate({value})"),
            DataType::Double => format!("StrToFloat({value})"),
            DataType::Binary(BinaryEncoding::Base64) => {
                format!("TNetEncoding.Base64.DecodeStringToBytes({value})")
            }
            DataType::Binary(BinaryEncoding::Hex) => format!("HexStrToBin({value})"),
            DataType::String => value,
            DataType::Time if pattern.is_some() => format!(
                "TimeOf(DecodeDateTime({}, '{}'))",
                value,
                pattern.unwrap_or_default(),
            ),
            DataType::Time => format!("TimeOf(ISO8601ToDate({value}))"),
            DataType::Uri => format!("TURI.Create({value})"),
            DataType::SmallInteger
            | DataType::ShortInteger
            | DataType::Integer
            | DataType::LongInteger
            | DataType::UnsignedSmallInteger
            | DataType::UnsignedShortInteger
            | DataType::UnsignedInteger
            | DataType::UnsignedLongInteger => {
                format!("StrToInt({value})")
            }
            _ => String::new(),
        }
    }

    fn generate_standard_type_to_xml(
        data_type: &DataType,
        variable_name: &String,
        xml_name: &String,
        pattern: &Option<String>,
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
