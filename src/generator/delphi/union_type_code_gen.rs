use std::io::{BufWriter, Write};

use crate::generator::{
    code_generator_trait::{CodeGenError, CodeGenOptions},
    types::{TypeAlias, UnionType},
};

use super::helper::Helper;

pub(crate) struct UnionTypeCodeGenerator {}

impl UnionTypeCodeGenerator {
    pub(crate) fn write_declarations<T: Write>(
        buffer: &mut BufWriter<T>,
        union_types: &[UnionType],
        options: &CodeGenOptions,
        indentation: usize,
    ) -> Result<(), CodeGenError> {
        if union_types.is_empty() {
            return Ok(());
        }

        buffer.write_fmt(format_args!(
            "{}{{$REGION 'Union Variants'}}\n",
            " ".repeat(indentation),
        ))?;
        for union_type in union_types {
            let enum_type_name = Self::get_variant_enum_type_name(&union_type.name, options);
            let variant_prefix = Helper::get_enum_variant_prefix(enum_type_name.as_str());

            buffer.write_fmt(format_args!(
                "{}{} = ({});\n",
                " ".repeat(indentation),
                enum_type_name,
                union_type
                    .variants
                    .iter()
                    .enumerate()
                    .map(|(i, _)| Self::get_variant_enum_variant_name(&variant_prefix, i))
                    .collect::<Vec<String>>()
                    .join(", ")
            ))?;
        }
        buffer.write_fmt(format_args!("{}{{$ENDREGION}}\n", " ".repeat(indentation)))?;

        buffer.write_all(b"\n")?;
        buffer.write_fmt(format_args!(
            "{}{{$REGION 'Union Types'}}\n",
            " ".repeat(indentation),
        ))?;
        for (i, union_type) in union_types.iter().enumerate() {
            Self::generate_declaration(buffer, union_type, options, indentation)?;

            if i < union_types.len() - 1 {
                buffer.write_all(b"\n")?;
            }
        }
        buffer.write_fmt(format_args!("{}{{$ENDREGION}}\n", " ".repeat(indentation)))?;

        buffer.write_all(b"\n")?;
        buffer.write_fmt(format_args!(
            "{}{{$REGION 'Union Types Helper'}}\n",
            " ".repeat(indentation),
        ))?;
        for (i, union_type) in union_types.iter().enumerate() {
            Self::generate_helper_declaration(buffer, union_type, options, indentation)?;

            if i < union_types.len() - 1 {
                buffer.write_all(b"\n")?;
            }
        }
        buffer.write_fmt(format_args!("{}{{$ENDREGION}}\n", " ".repeat(indentation)))?;

        Ok(())
    }

    pub(crate) fn write_implementations<T: Write>(
        buffer: &mut BufWriter<T>,
        union_types: &[UnionType],
        type_aliases: &[TypeAlias],
        options: &CodeGenOptions,
    ) -> Result<(), CodeGenError> {
        if union_types.is_empty() {
            return Ok(());
        }

        buffer.write_all(b"{$REGION 'Union Types Helper'}\n")?;
        for (i, union_type) in union_types.iter().enumerate() {
            Self::generate_helper_implementation(buffer, union_type, type_aliases, options)?;

            if i < union_types.len() - 1 {
                buffer.write_all(b"\n")?;
            }
        }
        buffer.write_all(b"{$ENDREGION}\n\n")?;

        Ok(())
    }

    fn generate_declaration<T: Write>(
        buffer: &mut BufWriter<T>,
        union_type: &UnionType,
        options: &CodeGenOptions,
        indentation: usize,
    ) -> Result<(), CodeGenError> {
        let enum_type_name = Self::get_variant_enum_type_name(&union_type.name, options);
        let variant_prefix = Helper::get_enum_variant_prefix(enum_type_name.as_str());

        buffer.write_fmt(format_args!(
            "{}{} = record",
            " ".repeat(indentation),
            Helper::as_type_name(&union_type.name, &options.type_prefix),
        ))?;
        buffer.write_all(b"\n")?;

        buffer.write_fmt(format_args!(
            "{}case Variant: {} of\n",
            " ".repeat(indentation + 2),
            enum_type_name,
        ))?;

        for (i, variant) in union_type.variants.iter().enumerate() {
            buffer.write_fmt(format_args!(
                "{}{}.{}: ({}: {});\n",
                " ".repeat(indentation + 4),
                enum_type_name,
                Self::get_variant_enum_variant_name(&variant_prefix, i),
                Helper::as_variable_name(variant.name.as_str()),
                Helper::get_datatype_language_representation(
                    &variant.data_type,
                    &options.type_prefix
                ),
            ))?;
        }

        buffer.write_fmt(format_args!("{}end;\n", " ".repeat(indentation)))?;

        Ok(())
    }

    fn generate_helper_declaration<T: Write>(
        buffer: &mut BufWriter<T>,
        union_type: &UnionType,
        options: &CodeGenOptions,
        indentation: usize,
    ) -> Result<(), CodeGenError> {
        buffer.write_fmt(format_args!(
            "{}{}Helper = record helper for {}\n",
            " ".repeat(indentation),
            Helper::as_type_name(&union_type.name, &options.type_prefix),
            Helper::as_type_name(&union_type.name, &options.type_prefix),
        ))?;

        if options.generate_from_xml {
            buffer.write_fmt(format_args!(
                "{}class function FromXml(node: IXMLNode): {}; static;\n",
                " ".repeat(indentation + 2),
                Helper::as_type_name(&union_type.name, &options.type_prefix),
            ))?;
        }

        if options.generate_from_xml && options.generate_to_xml {
            buffer.write_all(b"\n")?;
        }

        if options.generate_to_xml {
            buffer.write_fmt(format_args!(
                "{}function ToXmlValue: String;\n",
                " ".repeat(indentation + 2),
            ))?;
        }
        buffer.write_fmt(format_args!("{}end;\n", " ".repeat(indentation)))?;

        Ok(())
    }

    fn generate_helper_implementation<T: Write>(
        buffer: &mut BufWriter<T>,
        union_type: &UnionType,
        type_aliases: &[TypeAlias],
        options: &CodeGenOptions,
    ) -> Result<(), CodeGenError> {
        if options.generate_from_xml {
            buffer.write_fmt(format_args!(
                "class function {}Helper.FromXml(node: IXMLNode): {};\n",
                Helper::as_type_name(&union_type.name, &options.type_prefix),
                Helper::as_type_name(&union_type.name, &options.type_prefix),
            ))?;
            buffer.write_all(b"begin\n")?;
            buffer.write_all(b"  // TODO: Add implementation\n")?;
            buffer.write_all(b"end;\n")?;
        }

        if options.generate_from_xml && options.generate_to_xml {
            buffer.write_all(b"\n")?;
        }

        if options.generate_to_xml {
            let enum_type_name = Self::get_variant_enum_type_name(&union_type.name, options);
            let variant_prefix = Helper::get_enum_variant_prefix(enum_type_name.as_str());

            buffer.write_fmt(format_args!(
                "function {}Helper.ToXmlValue: String;\n",
                Helper::as_type_name(&union_type.name, &options.type_prefix),
            ))?;
            buffer.write_all(b"begin\n")?;
            buffer.write_fmt(format_args!("{}case Self.Variant of\n", " ".repeat(2)))?;
            for (i, variant) in union_type.variants.iter().enumerate() {
                let variable_name = Helper::as_variable_name(variant.name.as_str());

                match &variant.data_type {
                    crate::generator::types::DataType::Alias(n) => {
                        let Some((dt, pattern)) = Helper::get_alias_data_type(n.as_str(), type_aliases) else {
                            return Err(CodeGenError::MissingDataType(union_type.name.clone(), variable_name));
                        };

                        buffer.write_fmt(format_args!(
                            "{}{}.{}: Result := {};\n",
                            " ".repeat(4),
                            enum_type_name,
                            Self::get_variant_enum_variant_name(&variant_prefix, i),
                            Helper::get_variable_value_as_string(&dt, &variable_name, pattern,),
                        ))?;
                    }
                    crate::generator::types::DataType::Custom(_) => {
                        return Err(CodeGenError::ComplexTypeInSimpleTypeNotAllowed(
                            union_type.name.clone(),
                            variant.name.clone(),
                        ))
                    }
                    crate::generator::types::DataType::Enumeration(_) => {
                        buffer.write_fmt(format_args!(
                            "{}{}.{}: Result := {}.ToXmlValue;\n",
                            " ".repeat(4),
                            enum_type_name,
                            Self::get_variant_enum_variant_name(&variant_prefix, i),
                            variable_name,
                        ))?
                    }
                    crate::generator::types::DataType::List(_) => todo!(),
                    crate::generator::types::DataType::FixedSizeList(_, _) => todo!(),
                    crate::generator::types::DataType::Union(_) => todo!(),
                    _ => buffer.write_fmt(format_args!(
                        "{}{}.{}: Result := {};\n",
                        " ".repeat(4),
                        enum_type_name,
                        Self::get_variant_enum_variant_name(&variant_prefix, i),
                        Helper::get_variable_value_as_string(
                            &variant.data_type,
                            &variable_name,
                            None,
                        ),
                    ))?,
                }
            }
            buffer.write_fmt(format_args!("{}else Result := '';\n", " ".repeat(4)))?;
            buffer.write_fmt(format_args!("{}end;\n", " ".repeat(2)))?;
            buffer.write_all(b"end;\n")?;
        }

        Ok(())
    }

    fn get_variant_enum_type_name(name: &String, options: &CodeGenOptions) -> String {
        format!(
            "{}Variants",
            Helper::as_type_name(name, &options.type_prefix)
        )
    }

    fn get_variant_enum_variant_name(prefix: &String, index: usize) -> String {
        format!("{}Variant{}", prefix, index + 1)
    }
}
