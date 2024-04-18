use crate::generator::{
    code_generator_trait::CodeGenOptions,
    delphi::template_models::{
        Enumeration as TemplateEnumeration, EnumerationValue as TemplateEnumerationValue,
    },
    types::Enumeration,
};

use super::helper::Helper;

pub struct EnumCodeGenerator;

impl EnumCodeGenerator {
    pub fn build_template_models<'a>(
        enumerations: &'a [Enumeration],
        options: &'a CodeGenOptions,
    ) -> Vec<TemplateEnumeration<'a>> {
        enumerations
            .iter()
            .map(|e| {
                let prefix = Helper::get_enum_variant_prefix(&e.name);
                let documentations = e
                    .documentations
                    .iter()
                    .flat_map(|d| d.lines())
                    .collect::<Vec<&str>>();
                let line_per_variant = e.values.iter().any(|v| !v.documentations.is_empty());

                let values = e
                    .values
                    .iter()
                    .map(|v| {
                        let documentations = v
                            .documentations
                            .iter()
                            .flat_map(|d| d.lines())
                            .collect::<Vec<&str>>();

                        TemplateEnumerationValue {
                            variant_name: prefix.clone()
                                + Helper::first_char_uppercase(&v.variant_name).as_str(),
                            xml_value: v.xml_value.clone(),
                            documentations,
                        }
                    })
                    .collect::<Vec<TemplateEnumerationValue<'a>>>();

                TemplateEnumeration {
                    name: Helper::as_type_name(&e.name, &options.type_prefix),
                    qualified_name: e.qualified_name.clone(),
                    variant_prefix: prefix,
                    values,
                    documentations,
                    line_per_variant,
                }
            })
            .collect::<Vec<TemplateEnumeration<'a>>>()
    }
}
