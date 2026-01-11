#![allow(clippy::too_many_lines)]

use std::{fs::File, io::BufWriter, path::PathBuf, time::Instant};

pub mod generator;
pub mod parser;
mod type_registry;

use generator::{
    code_generator_trait::{CodeGenOptions, CodeGenerator},
    delphi::code_generator::DelphiCodeGenerator,
    internal_representation::InternalRepresentation,
};
use genphi_core::type_registry::TypeRegistry;
use parser::{types::ParsedData, xml::XmlParser};

use crate::parser::types::CustomTypeDefinition;

pub fn generate_xml(source: &[PathBuf], output_path: &PathBuf, options: CodeGenOptions) {
    let overall_instant = Instant::now();

    let output_file = match File::create(output_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Could not create output file due to following error: \"{e:?}\"");
            return;
        }
    };

    let mut parser = XmlParser::default();
    let mut type_registry = TypeRegistry::<CustomTypeDefinition>::new();

    let data: ParsedData = if source.len() == 1 {
        match parser.parse_file(source.first().unwrap(), &mut type_registry) {
            Ok(n) => n,
            Err(error) => {
                eprintln!("An error occured: {error}");
                return;
            }
        }
    } else {
        match parser.parse_files(source, &mut type_registry) {
            Ok(n) => n,
            Err(error) => {
                eprintln!("An error occured: {error}");
                return;
            }
        }
    };

    let internal_representation = InternalRepresentation::build(&data, &type_registry);

    let buffer = BufWriter::new(Box::new(output_file));
    let mut generator = DelphiCodeGenerator::new(
        buffer,
        options,
        internal_representation,
        data.documentations,
    );

    match generator.generate() {
        Ok(()) => {
            println!(
                "Completed successfully within {}ms",
                overall_instant.elapsed().as_millis(),
            );
        }
        Err(e) => {
            eprintln!("Failed to write output to file due to following error: \"{e:?}\"");
        }
    }
}
