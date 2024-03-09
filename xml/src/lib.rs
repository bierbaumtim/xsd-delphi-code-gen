#![allow(clippy::too_many_lines)]
use std::{fs::File, io::BufWriter, path::PathBuf, time::Instant};

pub mod generator;
mod parser;
mod type_registry;

use generator::{
    code_generator_trait::{CodeGenOptions, CodeGenerator},
    delphi::code_generator::DelphiCodeGenerator,
    internal_representation::InternalRepresentation,
};
use parser::{types::ParsedData, xml::XmlParser};
use type_registry::TypeRegistry;

pub fn generate_xml(source: &[PathBuf], output_path: &PathBuf, options: CodeGenOptions) {
    let instant = Instant::now();

    let output_file = match File::create(output_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Could not create output file due to following error: \"{e:?}\"");
            return;
        }
    };

    let mut parser = XmlParser::default();
    let mut type_registry = TypeRegistry::new();

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

    let elapsed_for_parse = instant.elapsed().as_millis();
    println!("Files parsed in {elapsed_for_parse}ms");

    let internal_representation = InternalRepresentation::build(&data, &type_registry);

    let elapsed_for_ir = instant
        .elapsed()
        .as_millis()
        .saturating_sub(elapsed_for_parse);
    println!("Internal Representation created in {elapsed_for_ir}ms");

    let buffer = BufWriter::new(Box::new(output_file));
    let mut generator = DelphiCodeGenerator::new(
        buffer,
        options,
        internal_representation,
        data.documentations,
    );

    match generator.generate() {
        Ok(()) => println!(
            "Completed successfully within {}ms",
            instant.elapsed().as_millis().saturating_sub(elapsed_for_ir),
        ),
        Err(e) => {
            eprintln!("Failed to write output to file due to following error: \"{e:?}\"");
        }
    }
}
