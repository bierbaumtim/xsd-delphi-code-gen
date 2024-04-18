use crate::generator::{
    code_generator_trait::CodeGenOptions,
    delphi::template_models::{
        UnionType as TemplateUnionType, UnionVariant as TemplateUnionVariant,
    },
    types::{DataType, Enumeration, TypeAlias, UnionType},
};

use super::helper::Helper;

/// Code generator for union types.
///
/// # Example
///
/// ## Input
///
/// ```rust
/// use xsd_types::generator::types::{DataType, UnionType, UnionTypeVariant};
///
/// let union_type = UnionType {
///   name: "UnionType".to_string(),
///   qualified_name: "UnionType".to_string(),
///   variants: vec![
///     UnionTypeVariant {
///       name: "Variant1".to_string(),
///       data_type: DataType::Boolean,
///     },
///     UnionTypeVariant {
///       name: "Variant2".to_string(),
///       data_type: DataType::String,
///     },
///   ],
/// };
/// ```
///
/// ## Output
///
/// ```delphi
/// {$REGION 'Union Types'}
/// /// <summary>
/// /// UnionType
/// /// </summary>
/// type TUnionType = record
///   type Variants = (Variant1, Variant2);
///   
///   case Variant: Variants of
///     Variant1: (Variant1: Boolean);
///     Variant2: (Variant2: string[255]);
/// end;
/// {$ENDREGION}
///
/// {$REGION 'Union Types Helper'}
/// type TUnionTypeHelper = record helper for TUnionType
///
/// end;
/// {$ENDREGION}
/// ```
pub struct UnionTypeCodeGenerator {}

impl UnionTypeCodeGenerator {
    pub(crate) fn build_template_models<'a>(
        union_types: &'a [UnionType],
        type_aliases: &'a [TypeAlias],
        enumerations: &[Enumeration],
        options: &'a CodeGenOptions,
    ) -> Vec<TemplateUnionType<'a>> {
        union_types
            .iter()
            .map(|u| {
                let variant_prefix = Self::get_enum_variant_prefix(&u.name, options);
                let documentations = u
                    .documentations
                    .iter()
                    .flat_map(|d| d.lines())
                    .collect::<Vec<&str>>();

                let variants = u
                    .variants
                    .iter()
                    .enumerate()
                    .map(|(i, v)| {
                        let variable_name = Helper::as_variable_name(&v.name);
                        let mut is_list_type = matches!(
                            v.data_type,
                            DataType::List(_) | DataType::FixedSizeList(_, _)
                        );
                        let mut is_inline_list = matches!(v.data_type, DataType::InlineList(_));
                        let mut use_to_xml_func =
                            matches!(v.data_type, DataType::Enumeration(_) | DataType::Union(_));
                        let mut value_as_str_repr = String::new();

                        match &v.data_type {
                            DataType::Alias(n) => {
                                if let Some((dt, pattern)) =
                                    Helper::get_alias_data_type(n.as_str(), type_aliases)
                                {
                                    match dt {
                                        DataType::Alias(_) => (),
                                        DataType::Custom(n) => {
                                            if enumerations.iter().any(|e| e.name == n) {
                                                use_to_xml_func = true;
                                            }
                                        }
                                        DataType::Enumeration(_) | DataType::Union(_) => {
                                            use_to_xml_func = true;
                                        }
                                        DataType::List(_) | DataType::FixedSizeList(_, _) => {
                                            is_list_type = true;
                                        }
                                        DataType::InlineList(lt) => {
                                            is_inline_list = true;
                                            value_as_str_repr =
                                                Helper::get_variable_value_as_string(
                                                    lt.as_ref(),
                                                    &format!("{variable_name}[I]"),
                                                    &pattern,
                                                );
                                        }
                                        _ => {
                                            value_as_str_repr =
                                                Helper::get_variable_value_as_string(
                                                    &v.data_type,
                                                    &variable_name,
                                                    &pattern,
                                                );
                                        }
                                    }
                                }
                            }
                            DataType::Custom(_)
                            | DataType::Enumeration(_)
                            | DataType::List(_)
                            | DataType::FixedSizeList(_, _)
                            | DataType::Union(_) => (),
                            DataType::InlineList(lt) => {
                                is_inline_list = true;
                                value_as_str_repr = Helper::get_variable_value_as_string(
                                    lt.as_ref(),
                                    &format!("{variable_name}[I]"),
                                    &None,
                                );
                            }
                            _ => {
                                value_as_str_repr = Helper::get_variable_value_as_string(
                                    &v.data_type,
                                    &variable_name,
                                    &None,
                                );
                            }
                        }

                        TemplateUnionVariant {
                            name: Self::get_variant_enum_variant_name(&variant_prefix, &v.name, i),
                            variable_name,
                            data_type_repr: match &v.data_type {
                                DataType::Alias(a) => {
                                    if let Some((dt, _)) =
                                        Helper::get_alias_data_type(a, type_aliases)
                                    {
                                        match dt {
                                            DataType::String => "string[255]".to_owned(),
                                            _ => Helper::get_datatype_language_representation(
                                                &v.data_type,
                                                &options.type_prefix,
                                            ),
                                        }
                                    } else {
                                        "Unknown".to_owned()
                                    }
                                }
                                DataType::InlineList(lt) => format!(
                                    "array[1..256] of {}",
                                    Helper::get_datatype_language_representation(
                                        lt.as_ref(),
                                        &options.type_prefix,
                                    ),
                                ),
                                _ => Helper::get_datatype_language_representation(
                                    &v.data_type,
                                    &options.type_prefix,
                                ),
                            },
                            use_to_xml_func,
                            is_inline_list,
                            is_list_type,
                            value_as_str_repr,
                        }
                    })
                    .collect::<Vec<TemplateUnionVariant>>();

                TemplateUnionType {
                    name: Helper::as_type_name(&u.name, &options.type_prefix),
                    qualified_name: &u.qualified_name,
                    documentations,
                    variants,
                }
            })
            .collect::<Vec<TemplateUnionType<'a>>>()
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
