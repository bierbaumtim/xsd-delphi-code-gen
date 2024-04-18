use crate::generator::{
    code_generator_trait::CodeGenOptions,
    delphi::template_models::TypeAlias as TemplateTypeAlias,
    types::{DataType, TypeAlias},
};

use super::helper::Helper;

/// Code generator for type aliases
///
/// # Example
///
/// ## Input
///
/// ```rust
/// use xsd_types::generator::types::{DataType, TypeAlias};
///
/// let type_aliases = vec![
///     TypeAlias {
///         pattern: None,
///         name: String::from("CustomString"),
///         qualified_name: String::from("CustomString"),
///         for_type: DataType::String,
///         documentations: Vec::new(),
///     },
///     TypeAlias {
///         pattern: None,
///         name: String::from("CustomIntList"),
///         qualified_name: String::from("CustomIntList"),
///         for_type: DataType::List(Box::new(DataType::Integer)),
///         documentations: Vec::new(),
///     },
/// ];
/// ```
///
/// ## Output
///
/// ```pascal
/// {$REGION 'Aliases'}
/// // XML Qualified Name: CustomString
/// TCustomString = String;
/// // XML Qualified Name: CustomIntList
/// TCustomIntList = TList<Integer>;
/// {$ENDREGION}
/// ```
pub struct TypeAliasCodeGenerator;

impl TypeAliasCodeGenerator {
    pub(crate) fn build_template_models<'a>(
        type_aliases: &'a [TypeAlias],
        options: &'a CodeGenOptions,
    ) -> Vec<TemplateTypeAlias<'a>> {
        type_aliases
            .iter()
            .filter_map(|a| {
                if matches!(&a.for_type, DataType::FixedSizeList(_, _)) {
                    return None;
                }

                let documentations = a
                    .documentations
                    .iter()
                    .flat_map(|d| d.lines())
                    .collect::<Vec<&str>>();

                Some(TemplateTypeAlias {
                    name: Helper::as_type_name(&a.name, &options.type_prefix),
                    qualified_name: &a.qualified_name,
                    pattern: &a.pattern,
                    data_type_repr: Helper::get_datatype_language_representation(
                        &a.for_type,
                        &options.type_prefix,
                    ),
                    documentations,
                })
            })
            .collect::<Vec<TemplateTypeAlias<'a>>>()
    }
}
