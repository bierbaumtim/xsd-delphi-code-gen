use crate::generator::{
    code_generator_trait::{CodeGenError, CodeGenOptions},
    delphi::template_models::{
        AttributeDeserializeVariable, ClassType as TemplateClassType, ElementDeserializeVariable,
        SerializeVariable as TemplateSerializeVariable, Variable as TemplateVariable,
    },
    internal_representation::DOCUMENT_NAME,
    types::{BinaryEncoding, ClassType, DataType, TypeAlias, Variable, XMLSource},
};

use super::helper::Helper;

impl DataType {
    /// Determines if the data type is a reference type.
    fn is_reference_type(&self, type_aliases: &[TypeAlias]) -> bool {
        match self {
            Self::Alias(n) => Helper::get_alias_data_type(n.as_str(), type_aliases)
                .map_or(true, |(dt, _)| dt.is_reference_type(type_aliases)),
            Self::Custom(_) | Self::List(_) | Self::InlineList(_) => true,
            Self::FixedSizeList(dt, _) => dt.as_ref().is_reference_type(type_aliases),
            _ => false,
        }
    }
}

impl Variable {
    fn is_optional(&self) -> bool {
        !self.required && !self.is_const && self.default_value.is_none()
    }

    fn needs_optional_wrapper(&self, type_aliases: &[TypeAlias]) -> bool {
        self.is_optional() && !self.data_type.is_reference_type(type_aliases)
    }
}

/// Code generator for classes.
pub struct ClassCodeGenerator;

impl ClassCodeGenerator {
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

    fn get_variable_initialization_code(
        name: &str,
        type_name: &str,
        is_required: bool,
        is_value_type: bool,
        default_value: &Option<String>,
    ) -> String {
        match (is_required, is_value_type, default_value) {
            (false, false, _) => format!("{name} := nil;"),
            (false, true, None) => format!("{name} := TNone<{type_name}>.Create;"),
            (true, false, _) => format!("{name} := {type_name}.Create;"),
            (true, true, None) => format!("{name} := Default({type_name});"),
            (_, true, Some(v)) => format!("{name} := {v};"),
        }
    }

    pub(crate) fn build_template_models<'a>(
        classes: &'a [ClassType],
        type_aliases: &'a [TypeAlias],
        options: &'a CodeGenOptions,
    ) -> Result<Vec<TemplateClassType<'a>>, CodeGenError> {
        classes
            .iter()
            .filter(|c| c.name != DOCUMENT_NAME)
            .map(|c| Self::build_class_template_model(c, type_aliases, options))
            .collect::<Result<Vec<TemplateClassType<'a>>, CodeGenError>>()
    }

    pub(crate) fn build_class_template_model<'a>(
        class_type: &'a ClassType,
        type_aliases: &'a [TypeAlias],
        options: &'a CodeGenOptions,
    ) -> Result<TemplateClassType<'a>, CodeGenError> {
        let needs_destructor = class_type
            .variables
            .iter()
            .any(|v| v.requires_free || !v.required);

        let documentations = class_type
            .documentations
            .iter()
            .flat_map(|d| d.lines())
            .collect::<Vec<&str>>();

        let constant_variables = class_type
            .variables
            .iter()
            .filter(|v| v.is_const)
            .map(|v| Self::build_standard_template_variable(v, options))
            .collect::<Vec<TemplateVariable>>();

        let optional_variables = class_type
            .variables
            .iter()
            .filter(|v| v.needs_optional_wrapper(type_aliases))
            .flat_map(|v| match &v.data_type {
                DataType::FixedSizeList(dt, size) => {
                    Self::build_fixed_size_list_template_variable(v, dt, *size, options)
                }
                _ => vec![Self::build_standard_template_variable(v, options)],
            })
            .collect::<Vec<TemplateVariable>>();

        let variables = Self::build_template_variables(class_type, type_aliases, options)?;

        let serialize_variables = Self::build_serialize_variables(class_type, type_aliases)?;

        let variable_initializer =
            Self::build_variable_initializer(class_type, type_aliases, options)?;

        let has_optional_element_variables = class_type
            .variables
            .iter()
            .any(|v| !v.required && !v.is_const && v.source == XMLSource::Element);

        let deserialize_element_variables =
            Self::build_deserialize_element_variables(class_type, type_aliases, options);

        let deserialize_attribute_variables =
            Self::build_deserialize_attribute_variables(class_type, type_aliases, options);

        Ok(TemplateClassType {
            name: Helper::as_type_name(&class_type.name, &options.type_prefix),
            qualified_name: &class_type.qualified_name,
            super_type: class_type
                .super_type
                .as_ref()
                .map(|(n, _)| Helper::as_type_name(n, &options.type_prefix)),
            has_optional_fields: !optional_variables.is_empty(),
            has_constant_fields: !constant_variables.is_empty(),
            documentations,
            needs_destructor,
            variables,
            constant_variables,
            optional_variables,
            serialize_variables,
            variable_initializer,
            has_optional_element_variables,
            deserialize_attribute_variables,
            deserialize_element_variables,
        })
    }

    fn build_template_variables<'a>(
        class_type: &'a ClassType,
        type_aliases: &'a [TypeAlias],
        options: &'a CodeGenOptions,
    ) -> Result<Vec<TemplateVariable<'a>>, CodeGenError> {
        let variables = class_type
            .variables
            .iter()
            .filter(|v| !v.is_const && !v.needs_optional_wrapper(type_aliases))
            .map(|v| match &v.data_type {
                DataType::Alias(n) => {
                    if let Some((data_type, _)) =
                        Helper::get_alias_data_type(n.as_str(), type_aliases)
                    {
                        let data_type_repr = if let DataType::InlineList(_) = data_type {
                            Helper::as_type_name(n, &options.type_prefix)
                        } else {
                            Helper::get_datatype_language_representation(
                                &v.data_type,
                                &options.type_prefix,
                            )
                        };

                        Ok(vec![TemplateVariable {
                            name: Helper::as_variable_name(&v.name),
                            xml_name: &v.xml_name,
                            default_value: &v.default_value,
                            required: v.required,
                            requires_free: v.requires_free,
                            data_type_repr,
                        }])
                    } else {
                        Err(CodeGenError::MissingDataType(
                            class_type.name.clone(),
                            Helper::as_variable_name(&v.name),
                        ))
                    }
                }
                DataType::FixedSizeList(dt, size) => Ok(
                    Self::build_fixed_size_list_template_variable(v, dt, *size, options),
                ),
                _ => Ok(vec![Self::build_standard_template_variable(v, options)]),
            })
            .collect::<Result<Vec<Vec<TemplateVariable>>, CodeGenError>>()?
            .into_iter()
            .flatten()
            .collect::<Vec<TemplateVariable<'a>>>();

        Ok(variables)
    }

    fn build_standard_template_variable<'a>(
        variable: &'a Variable,
        options: &'a CodeGenOptions,
    ) -> TemplateVariable<'a> {
        TemplateVariable {
            name: Helper::as_variable_name(&variable.name),
            xml_name: &variable.xml_name,
            data_type_repr: Helper::get_datatype_language_representation(
                &variable.data_type,
                &options.type_prefix,
            ),
            default_value: &variable.default_value,
            required: variable.required,
            requires_free: variable.requires_free,
        }
    }

    fn build_fixed_size_list_template_variable<'a>(
        variable: &'a Variable,
        data_type: &'a DataType,
        size: usize,
        options: &CodeGenOptions,
    ) -> Vec<TemplateVariable<'a>> {
        (1..size + 1)
            .map(|i| TemplateVariable {
                name: format!("{}{}", Helper::as_variable_name(&variable.name), i),
                xml_name: &variable.xml_name,
                data_type_repr: Helper::get_datatype_language_representation(
                    data_type,
                    &options.type_prefix,
                ),
                default_value: &variable.default_value,
                required: variable.required,
                requires_free: variable.requires_free,
            })
            .collect::<Vec<TemplateVariable>>()
    }

    fn build_serialize_variables<'a>(
        class_type: &'a ClassType,
        type_aliases: &'a [TypeAlias],
    ) -> Result<Vec<TemplateSerializeVariable<'a>>, CodeGenError> {
        let variables = class_type
            .variables
            .iter()
            .map(|v| {
                let variable_name = Helper::as_variable_name(&v.name);

                match &v.data_type {
                    DataType::Alias(name) => {
                        if let Some((data_type, pattern)) =
                            Helper::get_alias_data_type(name.as_str(), type_aliases)
                        {
                            let has_optional_wrapper = v.needs_optional_wrapper(type_aliases);

                            let variable_getter = match &data_type {
                                DataType::InlineList(_) => format!("{variable_name}[I]"),
                                _ if has_optional_wrapper => variable_name.clone() + ".Unwrap",
                                _ => variable_name.clone(),
                            };

                            let getter_data_type = match &data_type {
                                DataType::InlineList(lt) => lt,
                                _ => &data_type,
                            };

                            Ok(vec![TemplateSerializeVariable {
                                name: variable_name,
                                xml_name: &v.xml_name,
                                is_required: v.required,
                                is_class: false,
                                is_enum: false,
                                is_list: false,
                                is_inline_list: matches!(data_type, DataType::InlineList(_)),
                                from_xml_code: String::new(),
                                to_xml_code: Helper::get_variable_value_as_string(
                                    getter_data_type,
                                    &variable_getter,
                                    &pattern,
                                ),
                                has_optional_wrapper,
                            }])
                        } else {
                            Ok(vec![])
                        }
                    }
                    DataType::Enumeration(_) => Ok(vec![TemplateSerializeVariable {
                        name: variable_name,
                        xml_name: &v.xml_name,
                        is_required: v.required,
                        is_class: false,
                        is_enum: true,
                        is_list: false,
                        is_inline_list: false,
                        has_optional_wrapper: v.needs_optional_wrapper(type_aliases),
                        from_xml_code: String::new(),
                        to_xml_code: String::new(),
                    }]),
                    DataType::Custom(_) => Ok(vec![TemplateSerializeVariable {
                        name: variable_name,
                        xml_name: &v.xml_name,
                        is_required: v.required,
                        is_class: true,
                        is_enum: false,
                        is_list: false,
                        is_inline_list: false,
                        has_optional_wrapper: v.needs_optional_wrapper(type_aliases),
                        from_xml_code: String::new(),
                        to_xml_code: String::new(),
                    }]),
                    DataType::List(lt) => Ok(vec![TemplateSerializeVariable {
                        name: variable_name,
                        xml_name: &v.xml_name,
                        is_required: v.required,
                        is_class: matches!(**lt, DataType::Custom(_)),
                        is_enum: matches!(**lt, DataType::Enumeration(_)),
                        is_list: true,
                        is_inline_list: false,
                        has_optional_wrapper: v.needs_optional_wrapper(type_aliases),
                        from_xml_code: String::new(),
                        to_xml_code: Helper::get_variable_value_as_string(
                            lt,
                            &String::from("__Item"),
                            &None,
                        ),
                    }]),
                    DataType::FixedSizeList(dt, size) => Ok((1..size + 1)
                        .map(|i| TemplateSerializeVariable {
                            name: format!("{}{}", Helper::as_variable_name(&v.name), i),
                            xml_name: &v.xml_name,
                            is_required: v.required,
                            is_class: matches!(**dt, DataType::Custom(_)),
                            is_enum: matches!(**dt, DataType::Enumeration(_)),
                            is_list: false,
                            is_inline_list: false,
                            has_optional_wrapper: v.needs_optional_wrapper(type_aliases),
                            from_xml_code: String::new(),
                            to_xml_code: Helper::get_variable_value_as_string(
                                dt,
                                &format!("{}{}", Helper::as_variable_name(&v.name), i),
                                &None,
                            ),
                        })
                        .collect::<Vec<TemplateSerializeVariable>>()),
                    _ => {
                        let has_optional_wrapper = v.needs_optional_wrapper(type_aliases);

                        let variable_getter = if has_optional_wrapper {
                            variable_name.clone() + ".Unwrap"
                        } else {
                            variable_name.clone()
                        };

                        Ok(vec![TemplateSerializeVariable {
                            name: variable_name,
                            xml_name: &v.xml_name,
                            is_required: v.required,
                            is_class: false,
                            is_enum: false,
                            is_list: false,
                            is_inline_list: false,
                            from_xml_code: String::new(),
                            to_xml_code: Helper::get_variable_value_as_string(
                                &v.data_type,
                                &variable_getter,
                                &None,
                            ),
                            has_optional_wrapper,
                        }])
                    }
                }
            })
            .collect::<Result<Vec<Vec<TemplateSerializeVariable<'a>>>, CodeGenError>>()?
            .into_iter()
            .flatten()
            .collect::<Vec<TemplateSerializeVariable<'a>>>();

        Ok(variables)
    }

    fn build_variable_initializer<'a>(
        class_type: &'a ClassType,
        type_aliases: &'a [TypeAlias],
        options: &'a CodeGenOptions,
    ) -> Result<Vec<String>, CodeGenError> {
        let serialize_variables = class_type
            .variables
            .iter()
            .map(|v| {
                let variable_name = Helper::as_variable_name(&v.name);

                match &v.data_type {
                    DataType::Alias(name) => {
                        if let Some((data_type, _)) =
                            Helper::get_alias_data_type(name.as_str(), type_aliases)
                        {
                            Ok(vec![match data_type {
                                DataType::InlineList(_) => Self::get_variable_initialization_code(
                                    &variable_name,
                                    &Helper::get_datatype_language_representation(
                                        &data_type,
                                        &options.type_prefix,
                                    ),
                                    v.required,
                                    false,
                                    &v.default_value,
                                ),
                                _ => Self::get_variable_initialization_code(
                                    &variable_name,
                                    &Helper::as_type_name(name, &options.type_prefix),
                                    v.required,
                                    true,
                                    &v.default_value,
                                ),
                            }])
                        } else {
                            Ok(vec![])
                        }
                    }
                    DataType::Enumeration(name) => {
                        Ok(vec![Self::get_variable_initialization_code(
                            &variable_name,
                            &Helper::as_type_name(name, &options.type_prefix),
                            v.required,
                            true,
                            &v.default_value,
                        )])
                    }
                    DataType::Custom(name) => Ok(vec![Self::get_variable_initialization_code(
                        &variable_name,
                        &Helper::as_type_name(name, &options.type_prefix),
                        v.required,
                        false,
                        &v.default_value,
                    )]),
                    DataType::List(_) => Ok(vec![Self::get_variable_initialization_code(
                        &variable_name,
                        &Helper::get_datatype_language_representation(
                            &v.data_type,
                            &options.type_prefix,
                        ),
                        true,
                        false,
                        &v.default_value,
                    )]),
                    DataType::FixedSizeList(dt, size) => {
                        let rhs = match dt.as_ref() {
                            DataType::Alias(name) => {
                                if let Some((data_type, _)) =
                                    Helper::get_alias_data_type(name.as_str(), type_aliases)
                                {
                                    let type_name =
                                        Helper::as_type_name(name, &options.type_prefix);

                                    match data_type {
                                        DataType::Custom(_) => String::from("nil"),
                                        _ if v.required => format!("Default({type_name})"),
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

                                if v.required {
                                    format!("Default({type_name})")
                                } else {
                                    format!("TNone<{type_name}>.Create")
                                }
                            }
                            DataType::Custom(name) => {
                                if v.required {
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
                                    v.name.clone(),
                                ));
                            }
                            DataType::FixedSizeList(_, _) => {
                                return Err(CodeGenError::NestedFixedSizeList(
                                    class_type.name.clone(),
                                    v.name.clone(),
                                ));
                            }
                            _ => {
                                let lang_rep = Helper::get_datatype_language_representation(
                                    dt.as_ref(),
                                    &options.type_prefix,
                                );

                                if v.required {
                                    format!("Default({lang_rep})")
                                } else {
                                    format!("TNone<{lang_rep}>.Create")
                                }
                            }
                        };

                        Ok((1..size + 1)
                            .map(|i| format!("{variable_name}{i} := {rhs};"))
                            .collect::<Vec<String>>())
                    }
                    _ => Ok(vec![match &v.data_type {
                        DataType::Uri if v.required => {
                            format!("{variable_name} := TURI.Create('');")
                        }
                        DataType::Uri => format!("{variable_name} := TNone<TURI>.Create;"),
                        DataType::InlineList(_) => Self::get_variable_initialization_code(
                            &variable_name,
                            &Helper::get_datatype_language_representation(
                                &v.data_type,
                                &options.type_prefix,
                            ),
                            true,
                            false,
                            &v.default_value,
                        ),
                        _ => Self::get_variable_initialization_code(
                            &variable_name,
                            &Helper::get_datatype_language_representation(
                                &v.data_type,
                                &options.type_prefix,
                            ),
                            v.required,
                            true,
                            &v.default_value,
                        ),
                    }]),
                }
            })
            .collect::<Result<Vec<Vec<String>>, CodeGenError>>()?
            .into_iter()
            .flatten()
            .filter(|i| !i.is_empty())
            .collect::<Vec<String>>();

        Ok(serialize_variables)
    }

    fn build_deserialize_element_variables<'a>(
        class_type: &'a ClassType,
        type_aliases: &'a [TypeAlias],
        options: &'a CodeGenOptions,
    ) -> Vec<ElementDeserializeVariable<'a>> {
        class_type
            .variables
            .iter()
            .filter(|v| !v.is_const && v.source == XMLSource::Element)
            .filter_map(|v| {
                let variable_name = Helper::as_variable_name(&v.name);

                match &v.data_type {
                    DataType::Alias(name) => {
                        let (data_type, pattern) = Helper::get_alias_data_type(name, type_aliases)?;

                        let from_xml_code = match &data_type {
                            DataType::InlineList(item_type) => match item_type.as_ref() {
                                DataType::Alias(name) => {
                                    let (data_type, pattern) =
                                        Helper::get_alias_data_type(name, type_aliases)?;

                                    Self::generate_standard_type_from_xml(
                                        &data_type,
                                        "vPart".to_owned(),
                                        pattern,
                                    )
                                }
                                DataType::Enumeration(name) | DataType::Union(name) => {
                                    format!(
                                        "{}Helper.FromXmlValue(vPart)",
                                        Helper::as_type_name(name, &options.type_prefix)
                                    )
                                }
                                DataType::Custom(_)
                                | DataType::List(_)
                                | DataType::FixedSizeList(_, _)
                                | DataType::InlineList(_) => todo!(),
                                _ => Self::generate_standard_type_from_xml(
                                    &data_type,
                                    "vPart".to_owned(),
                                    None,
                                ),
                            },
                            _ => Self::generate_standard_type_from_xml(
                                &data_type,
                                format!("node.ChildNodes['{}'].Text", v.xml_name),
                                pattern,
                            ),
                        };

                        Some(ElementDeserializeVariable {
                            name: variable_name,
                            xml_name: &v.xml_name,
                            has_optional_wrapper: false,
                            is_required: v.required,
                            is_list: false,
                            is_inline_list: matches!(data_type, DataType::InlineList(_)),
                            is_fixed_size_list: false,
                            fixed_size_list_size: None,
                            data_type_repr: Helper::get_datatype_language_representation(
                                &data_type,
                                &options.type_prefix,
                            ),
                            from_xml_code,
                        })
                    }
                    DataType::Custom(name) => {
                        let type_name = Helper::as_type_name(name, &options.type_prefix);

                        let from_xml_code = match v.required {
                            true => {
                                format!("{}.FromXml(node.ChildNodes['{}'])", type_name, v.xml_name,)
                            }
                            false => format!("{type_name}.FromXml(vOptionalNode);"),
                        };

                        Some(ElementDeserializeVariable {
                            name: variable_name,
                            xml_name: &v.xml_name,
                            has_optional_wrapper: false,
                            is_required: v.required,
                            is_list: false,
                            is_inline_list: false,
                            is_fixed_size_list: false,
                            fixed_size_list_size: None,
                            data_type_repr: type_name,
                            from_xml_code,
                        })
                    }
                    DataType::Enumeration(name) => {
                        let type_name = Helper::as_type_name(name, &options.type_prefix);

                        let from_xml_code = match v.required {
                            true => format!(
                                "{}.FromXml(node.ChildNodes['{}'].Text)",
                                type_name, v.xml_name,
                            ),
                            false => format!("{type_name}.FromXml(vOptionalNode.Text)"),
                        };

                        Some(ElementDeserializeVariable {
                            name: variable_name,
                            xml_name: &v.xml_name,
                            has_optional_wrapper: v.needs_optional_wrapper(type_aliases),
                            is_required: v.required,
                            is_list: false,
                            is_inline_list: false,
                            is_fixed_size_list: false,
                            fixed_size_list_size: None,
                            data_type_repr: type_name,
                            from_xml_code,
                        })
                    }
                    DataType::FixedSizeList(item_type, size) => {
                        let from_xml_code = match item_type.as_ref() {
                            DataType::Alias(name) => {
                                let (data_type, pattern) =
                                    Helper::get_alias_data_type(name, type_aliases)?;

                                Self::generate_standard_type_from_xml(
                                    &data_type,
                                    format!("__{}Node.Text", variable_name),
                                    pattern,
                                )
                            }
                            DataType::Custom(name) => format!(
                                "{}.FromXml(__{}Node);",
                                Helper::as_type_name(name, &options.type_prefix),
                                variable_name
                            ),
                            DataType::Enumeration(name) => format!(
                                "{}.FromXmlValue(__{}Node.Text);",
                                Helper::as_type_name(name, &options.type_prefix),
                                variable_name
                            ),
                            _ => Self::generate_standard_type_from_xml(
                                item_type,
                                format!("__{}Node.Text", variable_name),
                                None,
                            ),
                        };

                        Some(ElementDeserializeVariable {
                            name: variable_name,
                            xml_name: &v.xml_name,
                            has_optional_wrapper: false,
                            is_required: v.required,
                            is_list: false,
                            is_inline_list: false,
                            is_fixed_size_list: true,
                            fixed_size_list_size: Some(*size),
                            data_type_repr: Helper::get_datatype_language_representation(
                                item_type,
                                &options.type_prefix,
                            ),
                            from_xml_code,
                        })
                    }
                    DataType::List(item_type) => {
                        let from_xml_code = match item_type.as_ref() {
                            DataType::Alias(name) => {
                                let (data_type, pattern) =
                                    Helper::get_alias_data_type(name, type_aliases)?;

                                Self::generate_standard_type_from_xml(
                                    &data_type,
                                    format!("__{}Node.Text", variable_name),
                                    pattern,
                                )
                            }
                            DataType::Custom(name) => format!(
                                "{}.FromXml(__{}Node)",
                                Helper::as_type_name(name, &options.type_prefix),
                                variable_name
                            ),
                            DataType::Enumeration(name) => format!(
                                "{}.FromXmlValue(__{}Node.Text)",
                                Helper::as_type_name(name, &options.type_prefix),
                                variable_name
                            ),
                            _ => Self::generate_standard_type_from_xml(
                                item_type,
                                format!("__{}Node.Text", variable_name),
                                None,
                            ),
                        };

                        Some(ElementDeserializeVariable {
                            name: variable_name,
                            xml_name: &v.xml_name,
                            has_optional_wrapper: false,
                            is_required: v.required,
                            is_list: true,
                            is_inline_list: false,
                            is_fixed_size_list: false,
                            fixed_size_list_size: None,
                            data_type_repr: Helper::get_datatype_language_representation(
                                &v.data_type,
                                &options.type_prefix,
                            ),
                            from_xml_code,
                        })
                    }
                    DataType::InlineList(item_type) => {
                        let from_xml_code = match item_type.as_ref() {
                            DataType::Alias(name) => {
                                let (data_type, pattern) =
                                    Helper::get_alias_data_type(name, type_aliases)?;

                                Self::generate_standard_type_from_xml(
                                    &data_type,
                                    "vPart".to_owned(),
                                    pattern,
                                )
                            }
                            DataType::Enumeration(name) | DataType::Union(name) => format!(
                                "{}Helper.FromXmlValue(vPart)",
                                Helper::as_type_name(name, &options.type_prefix)
                            ),
                            DataType::Custom(_)
                            | DataType::List(_)
                            | DataType::FixedSizeList(_, _)
                            | DataType::InlineList(_) => todo!(),
                            _ => Self::generate_standard_type_from_xml(
                                item_type,
                                "vPart".to_owned(),
                                None,
                            ),
                        };

                        Some(ElementDeserializeVariable {
                            name: variable_name,
                            xml_name: &v.xml_name,
                            has_optional_wrapper: false,
                            is_required: v.required,
                            is_list: false,
                            is_inline_list: true,
                            is_fixed_size_list: false,
                            fixed_size_list_size: None,
                            data_type_repr: Helper::get_datatype_language_representation(
                                &v.data_type,
                                &options.type_prefix,
                            ),
                            from_xml_code,
                        })
                    }
                    _ => Some(ElementDeserializeVariable {
                        name: variable_name,
                        xml_name: &v.xml_name,
                        has_optional_wrapper: v.needs_optional_wrapper(type_aliases),
                        is_required: v.required,
                        is_list: false,
                        is_inline_list: false,
                        is_fixed_size_list: false,
                        fixed_size_list_size: None,
                        data_type_repr: Helper::get_datatype_language_representation(
                            &v.data_type,
                            &options.type_prefix,
                        ),
                        from_xml_code: match v.required {
                            true => Self::generate_standard_type_from_xml(
                                &v.data_type,
                                format!("node.ChildNodes['{}'].Text", v.xml_name),
                                None,
                            ),
                            false => Self::generate_standard_type_from_xml(
                                &v.data_type,
                                "vOptionalNode.Text".to_owned(),
                                None,
                            ),
                        },
                    }),
                }
            })
            .collect::<Vec<ElementDeserializeVariable>>()
    }

    fn build_deserialize_attribute_variables<'a>(
        class_type: &'a ClassType,
        type_aliases: &'a [TypeAlias],
        options: &'a CodeGenOptions,
    ) -> Vec<AttributeDeserializeVariable<'a>> {
        class_type
            .variables
            .iter()
            .filter(|v| !v.is_const && v.source == XMLSource::Attribute)
            .filter_map(|v| {
                let (data_type, pattern) = match &v.data_type {
                    DataType::Alias(name) => Helper::get_alias_data_type(name, type_aliases)?,
                    _ => (v.data_type.clone(), None),
                };

                Some(AttributeDeserializeVariable {
                    name: Helper::as_variable_name(&v.name),
                    xml_name: &v.xml_name,
                    has_optional_wrapper: v.needs_optional_wrapper(type_aliases),
                    from_xml_code_available: Self::generate_standard_type_from_xml(
                        &data_type,
                        format!("node.Attributes['{}']", v.xml_name),
                        pattern,
                    ),
                    from_xml_code_missing: match (v.required, &v.default_value) {
                        (false, None) => {
                            let lang_rep = Helper::get_datatype_language_representation(
                                &data_type,
                                &options.type_prefix,
                            );

                            format!("TNone<{lang_rep}>.Create")
                        }
                        (true, None) => {
                            format!(
                                "raise Exception.Create('Required attribute \"{}\" is missing');",
                                v.xml_name
                            )
                        }
                        (_, Some(default_value)) => default_value.clone(),
                    },
                })
            })
            .collect::<Vec<AttributeDeserializeVariable>>()
    }
}

#[cfg(test)]
mod tests {
    // use pretty_assertions::assert_eq;

    // use super::*;

    // TODO: Write Test
}
