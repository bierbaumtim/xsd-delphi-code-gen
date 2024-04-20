use std::io::{BufWriter, Write};
use tera::{Context, Tera};

use crate::generator::{
    code_generator_trait::{CodeGenError, CodeGenOptions, CodeGenerator},
    internal_representation::InternalRepresentation,
    types::{BinaryEncoding, DataType},
};

use super::{
    alias_code_gen::TypeAliasCodeGenerator, class_code_gen::ClassCodeGenerator,
    code_writer::CodeWriter, enum_code_gen::EnumCodeGenerator,
    union_type_code_gen::UnionTypeCodeGenerator,
};

/// The Delphi code generator.
///
/// This struct is used to generate Delphi code from the internal representation.
///
/// # Example
///
/// ```rust
/// use std::io::BufWriter;
///
/// use xsd_codegen::generator::{
///     code_generator_trait::CodeGenOptions,
///     internal_representation::InternalRepresentation,
///     delphi::DelphiCodeGenerator,
/// };
///
/// let options = CodeGenOptions {
///     unit_name: "TestUnit".to_string(),
///     ..CodeGenOptions::default()
/// };
///
/// let internal_representation = InternalRepresentation::default();
///
/// let mut code_generator = DelphiCodeGenerator::new(
///     BufWriter::new(Vec::new()),
///     options,
///     internal_representation,
///     Vec::new(),
/// );
///
/// code_generator.generate().unwrap();
/// ```
pub struct DelphiCodeGenerator<T: Write> {
    writer: CodeWriter<T>,
    options: CodeGenOptions,
    internal_representation: InternalRepresentation,
    documentations: Vec<String>,
    generate_date_time_helper: bool,
    generate_hex_binary_helper: bool,
}

impl<T: Write> DelphiCodeGenerator<T> {
    #[inline]
    fn setup_tera(&self) -> Result<Tera, CodeGenError> {
        let macros_template_str = include_str!("templates/macros.pas");
        let template_str = include_str!("templates/models.pas");

        let mut tera = Tera::default();
        if let Err(e) = tera.add_raw_templates(vec![
            ("macros.pas", macros_template_str),
            ("models.pas", template_str),
        ]) {
            eprintln!("Failed to load templates due to {:?}", e);

            return Err(CodeGenError::TemplateEngineError(format!(
                "Failed to load templates due to {:?}",
                e
            )));
        }

        Ok(tera)
    }

    #[inline]
    fn build_tera_context(&self) -> Result<Context, CodeGenError> {
        let mut models_context = Context::new();
        models_context.insert("unitName", &self.options.unit_name);
        models_context.insert("crate_version", env!("CARGO_PKG_VERSION"));
        models_context.insert("gen_from_xml", &self.options.generate_from_xml);
        models_context.insert("gen_to_xml", &self.options.generate_to_xml);
        models_context.insert("gen_datetime_helper", &self.generate_date_time_helper);
        models_context.insert("gen_hex_binary_helper", &self.generate_hex_binary_helper);

        // Add calculated fields
        let gen_bool_consts = self.internal_representation.classes.iter().any(|c| {
            c.variables
                .iter()
                .any(|v| matches!(v.data_type, DataType::Boolean))
        });
        models_context.insert("gen_bool_consts", &gen_bool_consts);

        models_context.insert(
            "documentations",
            &self
                .documentations
                .iter()
                .flat_map(|s| s.lines())
                .collect::<Vec<&str>>(),
        );
        models_context.insert(
            "document",
            &ClassCodeGenerator::build_class_template_model(
                &self.internal_representation.document,
                &self.internal_representation.types_aliases,
                &self.options,
            )?,
        );
        models_context.insert(
            "classes",
            &ClassCodeGenerator::build_template_models(
                &self.internal_representation.classes,
                &self.internal_representation.types_aliases,
                &self.options,
            )?,
        );
        models_context.insert(
            "enumerations",
            &EnumCodeGenerator::build_template_models(
                &self.internal_representation.enumerations,
                &self.options,
            ),
        );
        models_context.insert(
            "type_aliases",
            &TypeAliasCodeGenerator::build_template_models(
                &self.internal_representation.types_aliases,
                &self.options,
            ),
        );
        models_context.insert(
            "union_types",
            &UnionTypeCodeGenerator::build_template_models(
                &self.internal_representation.union_types,
                &self.internal_representation.types_aliases,
                &self.internal_representation.enumerations,
                &self.options,
            ),
        );

        Ok(models_context)
    }
}

impl<T> CodeGenerator<T> for DelphiCodeGenerator<T>
where
    T: Write,
{
    fn new(
        buffer: BufWriter<T>,
        options: CodeGenOptions,
        internal_representation: InternalRepresentation,
        documentations: Vec<String>,
    ) -> Self {
        Self {
            writer: CodeWriter { buffer },
            options,
            documentations,
            generate_date_time_helper: internal_representation.classes.iter().any(|c| {
                c.variables.iter().any(|v| {
                    matches!(
                        &v.data_type,
                        DataType::DateTime | DataType::Date | DataType::Time
                    )
                })
            }) || internal_representation.types_aliases.iter().any(
                |a| {
                    matches!(
                        &a.for_type,
                        DataType::DateTime | DataType::Date | DataType::Time
                    )
                },
            ),
            generate_hex_binary_helper: internal_representation.classes.iter().any(|c| {
                c.variables
                    .iter()
                    .any(|v| matches!(&v.data_type, DataType::Binary(BinaryEncoding::Hex)))
            }) || internal_representation
                .types_aliases
                .iter()
                .any(|a| matches!(&a.for_type, DataType::Binary(BinaryEncoding::Hex))),
            internal_representation,
        }
    }

    fn generate(&mut self) -> Result<(), CodeGenError> {
        let tera = self.setup_tera()?;
        let models_context = self.build_tera_context()?;

        match tera.render_to("models.pas", &models_context, &mut self.writer.buffer) {
            Ok(_) => {}
            Err(e) => {
                return Err(CodeGenError::TemplateEngineError(format!(
                    "Failed to render model template due to {:?}",
                    e
                )));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // use pretty_assertions::assert_eq;

    // use super::*;

    // TODO: Write Test
}
