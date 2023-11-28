use std::io::Write;

use crate::generator::{
    code_generator_trait::{CodeGenError, CodeGenOptions},
    types::{DataType, Enumeration, TypeAlias, UnionType},
};

use super::{
    code_writer::{CodeWriter, FunctionType},
    helper::Helper,
};

pub(crate) struct UnionTypeCodeGenerator {}

impl UnionTypeCodeGenerator {
    const ENUM_NAME: &'static str = "Variants";

    pub(crate) fn write_declarations<T: Write>(
        writer: &mut CodeWriter<T>,
        union_types: &[UnionType],
        type_aliases: &[TypeAlias],
        options: &CodeGenOptions,
        indentation: usize,
    ) -> Result<(), CodeGenError> {
        if union_types.is_empty() {
            return Ok(());
        }

        writer.writeln("{$REGION 'Union Types'}", Some(indentation))?;
        for (i, union_type) in union_types.iter().enumerate() {
            Self::generate_declaration(writer, union_type, type_aliases, options, indentation)?;

            if i < union_types.len() - 1 {
                writer.newline()?;
            }
        }
        writer.writeln("{$ENDREGION}", Some(indentation))?;

        writer.newline()?;
        writer.writeln("{$REGION 'Union Types Helper'}", Some(indentation))?;
        for (i, union_type) in union_types.iter().enumerate() {
            Self::generate_helper_declaration(writer, union_type, options, indentation)?;

            if i < union_types.len() - 1 {
                writer.newline()?;
            }
        }
        writer.writeln("{$ENDREGION}", Some(indentation))?;

        Ok(())
    }

    pub(crate) fn write_implementations<T: Write>(
        writer: &mut CodeWriter<T>,
        union_types: &[UnionType],
        enumerations: &[Enumeration],
        type_aliases: &[TypeAlias],
        options: &CodeGenOptions,
    ) -> Result<(), CodeGenError> {
        if union_types.is_empty() {
            return Ok(());
        }

        writer.writeln("{$REGION 'Union Types Helper'}", None)?;
        for (i, union_type) in union_types.iter().enumerate() {
            Self::generate_helper_implementation(
                writer,
                union_type,
                enumerations,
                type_aliases,
                options,
            )?;

            if i < union_types.len() - 1 {
                writer.newline()?;
            }
        }
        writer.writeln("{$ENDREGION}", None)?;
        writer.newline()?;

        Ok(())
    }

    fn generate_declaration<T: Write>(
        writer: &mut CodeWriter<T>,
        union_type: &UnionType,
        type_aliases: &[TypeAlias],
        options: &CodeGenOptions,
        indentation: usize,
    ) -> Result<(), CodeGenError> {
        let variant_prefix = Self::get_enum_variant_prefix(&union_type.name, options);

        writer.write_documentation(&union_type.documentations, Some(indentation))?;
        Helper::write_qualified_name_comment(
            writer,
            &union_type.qualified_name,
            Some(indentation),
        )?;

        writer.writeln_fmt(
            format_args!(
                "{} = record",
                Helper::as_type_name(&union_type.name, &options.type_prefix),
            ),
            Some(indentation),
        )?;
        writer.writeln_fmt(
            format_args!(
                "type {} = ({});",
                Self::ENUM_NAME,
                union_type
                    .variants
                    .iter()
                    .enumerate()
                    .map(|(i, u)| Self::get_variant_enum_variant_name(&variant_prefix, &u.name, i))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Some(indentation + 2),
        )?;
        writer.newline()?;

        writer.writeln_fmt(
            format_args!("case Variant: {} of", Self::ENUM_NAME),
            Some(indentation + 2),
        )?;

        for (i, variant) in union_type.variants.iter().enumerate() {
            match &variant.data_type {
                DataType::Alias(a) => {
                    if let Some((dt, _)) = Helper::get_alias_data_type(a, type_aliases) {
                        match dt {
                            DataType::String => {
                                writer.writeln_fmt(
                                    format_args!(
                                        "{}.{}: ({}: string[255]);",
                                        Self::ENUM_NAME,
                                        Self::get_variant_enum_variant_name(
                                            &variant_prefix,
                                            &variant.name,
                                            i
                                        ),
                                        Helper::as_variable_name(variant.name.as_str()),
                                    ),
                                    Some(indentation + 4),
                                )?;
                            }
                            _ => {
                                writer.writeln_fmt(
                                    format_args!(
                                        "{}.{}: ({}: {});",
                                        Self::ENUM_NAME,
                                        Self::get_variant_enum_variant_name(
                                            &variant_prefix,
                                            &variant.name,
                                            i
                                        ),
                                        Helper::as_variable_name(variant.name.as_str()),
                                        Helper::get_datatype_language_representation(
                                            &variant.data_type,
                                            &options.type_prefix
                                        ),
                                    ),
                                    Some(indentation + 4),
                                )?;
                            }
                        }
                    }
                }
                DataType::InlineList(lt) => {
                    writer.writeln_fmt(
                        format_args!(
                            "{}.{}: ({}: array[1..256] of {});",
                            Self::ENUM_NAME,
                            Self::get_variant_enum_variant_name(&variant_prefix, &variant.name, i),
                            Helper::as_variable_name(variant.name.as_str()),
                            Helper::get_datatype_language_representation(
                                lt.as_ref(),
                                &options.type_prefix
                            ),
                        ),
                        Some(indentation + 4),
                    )?;
                }
                _ => {
                    writer.writeln_fmt(
                        format_args!(
                            "{}.{}: ({}: {});",
                            Self::ENUM_NAME,
                            Self::get_variant_enum_variant_name(&variant_prefix, &variant.name, i),
                            Helper::as_variable_name(variant.name.as_str()),
                            Helper::get_datatype_language_representation(
                                &variant.data_type,
                                &options.type_prefix
                            ),
                        ),
                        Some(indentation + 4),
                    )?;
                }
            }
        }

        writer.writeln("end;", Some(indentation))?;

        Ok(())
    }

    fn generate_helper_declaration<T: Write>(
        writer: &mut CodeWriter<T>,
        union_type: &UnionType,
        options: &CodeGenOptions,
        indentation: usize,
    ) -> Result<(), CodeGenError> {
        writer.writeln_fmt(
            format_args!(
                "{}Helper = record helper for {}",
                Helper::as_type_name(&union_type.name, &options.type_prefix),
                Helper::as_type_name(&union_type.name, &options.type_prefix),
            ),
            Some(indentation),
        )?;

        if options.generate_from_xml {
            writer.write_function_declaration(
                FunctionType::Function(Helper::as_type_name(
                    &union_type.name,
                    &options.type_prefix,
                )),
                "FromXml",
                Some(vec![("node", "IXMLNode")]),
                true,
                None,
                indentation + 2,
            )?;
        }

        if options.generate_from_xml && options.generate_to_xml {
            writer.newline()?;
        }

        if options.generate_to_xml {
            writer.writeln("function ToXmlValue: String;", Some(indentation + 2))?;
        }
        writer.writeln("end;", Some(indentation))?;

        Ok(())
    }

    fn generate_helper_implementation<T: Write>(
        writer: &mut CodeWriter<T>,
        union_type: &UnionType,
        enumerations: &[Enumeration],
        type_aliases: &[TypeAlias],
        options: &CodeGenOptions,
    ) -> Result<(), CodeGenError> {
        if options.generate_from_xml {
            writer.writeln_fmt(
                format_args!(
                    "class function {}Helper.FromXml(node: IXMLNode): {};",
                    Helper::as_type_name(&union_type.name, &options.type_prefix),
                    Helper::as_type_name(&union_type.name, &options.type_prefix),
                ),
                None,
            )?;
            writer.writeln("begin", None)?;
            writer.writeln("// TODO: CodeGen for this is currently not supported. Manual implementation required", Some(2))?;
            writer.writeln("end;", None)?;
        }

        if options.generate_from_xml && options.generate_to_xml {
            writer.newline()?;
        }

        if options.generate_to_xml {
            let variant_prefix = Self::get_enum_variant_prefix(&union_type.name, options);

            writer.writeln_fmt(
                format_args!(
                    "function {}Helper.ToXmlValue: String;\n",
                    Helper::as_type_name(&union_type.name, &options.type_prefix),
                ),
                None,
            )?;
            writer.writeln("begin", None)?;
            writer.writeln("case Self.Variant of", Some(2))?;
            for (i, variant) in union_type.variants.iter().enumerate() {
                let variable_name = Helper::as_variable_name(variant.name.as_str());

                match &variant.data_type {
                    crate::generator::types::DataType::Alias(n) => {
                        let Some((dt, pattern)) = Helper::get_alias_data_type(n.as_str(), type_aliases) else {
                            return Err(CodeGenError::MissingDataType(union_type.name.clone(), variable_name));
                        };

                        match dt {
                            crate::generator::types::DataType::Alias(_) => (),
                            crate::generator::types::DataType::Custom(n) => {
                                if enumerations.iter().any(|e| e.name == n) {
                                    writer.writeln_fmt(
                                        format_args!(
                                            "{}.{}: Result := {}.ToXmlValue;",
                                            Self::ENUM_NAME,
                                            Self::get_variant_enum_variant_name(
                                                &variant_prefix,
                                                &variant.name,
                                                i
                                            ),
                                            variable_name,
                                        ),
                                        Some(4),
                                    )?;
                                } else {
                                    return Err(CodeGenError::ComplexTypeInSimpleTypeNotAllowed(
                                        union_type.name.clone(),
                                        variant.name.clone(),
                                    ));
                                }
                            }
                            crate::generator::types::DataType::Enumeration(_) => writer
                                .writeln_fmt(
                                    format_args!(
                                        "{}.{}: Result := {}.ToXmlValue;",
                                        Self::ENUM_NAME,
                                        Self::get_variant_enum_variant_name(
                                            &variant_prefix,
                                            &variant.name,
                                            i
                                        ),
                                        variable_name,
                                    ),
                                    Some(4),
                                )?,
                            crate::generator::types::DataType::List(_)
                            | crate::generator::types::DataType::FixedSizeList(_, _) => {
                                writer.writeln_fmt(format_args!(
                                    "{}.{}: Result := ''; // TODO: CodeGen for this type is currently not supported. Manual implementation required",
                                    Self::ENUM_NAME,
                                    Self::get_variant_enum_variant_name(&variant_prefix, &variant.name, i),
                                ), Some(4))?;
                            }
                            crate::generator::types::DataType::InlineList(lt) => {
                                writer.writeln_fmt(
                                    format_args!(
                                        "{}.{}: begin",
                                        Self::ENUM_NAME,
                                        Self::get_variant_enum_variant_name(
                                            &variant_prefix,
                                            &variant.name,
                                            i
                                        )
                                    ),
                                    Some(4),
                                )?;
                                writer.writeln("Result := '';", Some(6))?;
                                writer.newline()?;
                                writer.writeln_fmt(
                                    format_args!(
                                        "for var I := Low({variable_name}) to High({variable_name}) do begin"
                                    ),
                                    Some(6),
                                )?;
                                writer.writeln_fmt(
                                    format_args!(
                                        "Result := Result + {};",
                                        Helper::get_variable_value_as_string(
                                            lt.as_ref(),
                                            &format!("{variable_name}[I]"),
                                            &None
                                        )
                                    ),
                                    Some(8),
                                )?;
                                writer.newline()?;
                                writer.writeln_fmt(
                                    format_args!("if I < High({variable_name}) then begin"),
                                    Some(8),
                                )?;
                                writer.writeln("Result := Result + ' ';", Some(10))?;
                                writer.writeln("end;", Some(8))?;
                                writer.writeln("end;", Some(6))?;
                                writer.writeln("end;", Some(4))?;
                            }
                            crate::generator::types::DataType::Union(n) => {
                                writer.writeln_fmt(
                                    format_args!(
                                        "{}.{}: Result := {}.ToXmlValue;",
                                        Self::ENUM_NAME,
                                        Self::get_variant_enum_variant_name(
                                            &variant_prefix,
                                            &variant.name,
                                            i
                                        ),
                                        Helper::as_type_name(&n, &options.type_prefix),
                                    ),
                                    Some(4),
                                )?;
                            }
                            _ => writer.writeln_fmt(
                                format_args!(
                                    "{}.{}: Result := {};",
                                    Self::ENUM_NAME,
                                    Self::get_variant_enum_variant_name(
                                        &variant_prefix,
                                        &variant.name,
                                        i
                                    ),
                                    Helper::get_variable_value_as_string(
                                        &dt,
                                        &variable_name,
                                        &pattern
                                    ),
                                ),
                                Some(4),
                            )?,
                        }
                    }
                    crate::generator::types::DataType::Custom(_) => {
                        return Err(CodeGenError::ComplexTypeInSimpleTypeNotAllowed(
                            union_type.name.clone(),
                            variant.name.clone(),
                        ))
                    }
                    crate::generator::types::DataType::Enumeration(_) => {
                        writer.writeln_fmt(
                            format_args!(
                                "{}.{}: Result := {}.ToXmlValue;",
                                Self::ENUM_NAME,
                                Self::get_variant_enum_variant_name(
                                    &variant_prefix,
                                    &variant.name,
                                    i
                                ),
                                variable_name,
                            ),
                            Some(4),
                        )?;
                    }
                    crate::generator::types::DataType::List(_)
                    | crate::generator::types::DataType::FixedSizeList(_, _) => {
                        writer.writeln_fmt(format_args!(
                            "{}.{}: Result := ''; // TODO: CodeGen for this type is currently not supported. Manual implementation required",
                            Self::ENUM_NAME,
                            Self::get_variant_enum_variant_name(&variant_prefix, &variant.name, i),
                        ), Some(4))?;
                    }
                    crate::generator::types::DataType::InlineList(lt) => {
                        writer.writeln_fmt(
                            format_args!(
                                "{}.{}: begin",
                                Self::ENUM_NAME,
                                Self::get_variant_enum_variant_name(
                                    &variant_prefix,
                                    &variant.name,
                                    i
                                )
                            ),
                            Some(4),
                        )?;
                        writer.writeln("Result := '';", Some(6))?;
                        writer.newline()?;
                        writer.writeln_fmt(format_args!("for var I := Low({variable_name}) to High({variable_name}) do begin"), Some(6))?;
                        writer.writeln_fmt(
                            format_args!(
                                "Result := Result + {};",
                                Helper::get_variable_value_as_string(
                                    lt.as_ref(),
                                    &format!("{variable_name}[I]"),
                                    &None
                                )
                            ),
                            Some(8),
                        )?;
                        writer.newline()?;
                        writer.writeln_fmt(
                            format_args!("if I < High({variable_name}) then begin"),
                            Some(8),
                        )?;
                        writer.writeln("Result := Result + ' ';", Some(10))?;
                        writer.writeln("end;", Some(8))?;
                        writer.writeln("end;", Some(6))?;
                        writer.writeln("end;", Some(4))?;
                    }
                    crate::generator::types::DataType::Union(n) => {
                        writer.writeln_fmt(
                            format_args!(
                                "{}.{}: Result := {}.ToXmlValue;",
                                Self::ENUM_NAME,
                                Self::get_variant_enum_variant_name(
                                    &variant_prefix,
                                    &variant.name,
                                    i
                                ),
                                Helper::as_type_name(n, &options.type_prefix),
                            ),
                            Some(4),
                        )?;
                    }
                    _ => writer.writeln_fmt(
                        format_args!(
                            "{}.{}: Result := {};",
                            Self::ENUM_NAME,
                            Self::get_variant_enum_variant_name(&variant_prefix, &variant.name, i),
                            Helper::get_variable_value_as_string(
                                &variant.data_type,
                                &variable_name,
                                &None,
                            ),
                        ),
                        Some(4),
                    )?,
                }
            }
            writer.writeln("end;", Some(2))?;
            writer.writeln("end;", None)?;
        }

        Ok(())
    }

    fn get_enum_variant_prefix(name: &String, options: &CodeGenOptions) -> String {
        let enum_type_name = format!(
            "{}Variants",
            Helper::as_type_name(name, &options.type_prefix)
        );

        Helper::get_enum_variant_prefix(enum_type_name.as_str())
    }

    fn get_variant_enum_variant_name(prefix: &String, name: &String, index: usize) -> String {
        if name.is_empty() {
            format!("{}{}", prefix, index + 1)
        } else {
            format!("{}{}", prefix, Helper::first_char_uppercase(name))
        }
    }
}
