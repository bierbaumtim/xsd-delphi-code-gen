use std::{fs::File, io::BufWriter, path::PathBuf, time::Instant};

use clap::{Parser, ValueEnum};

mod generator;
mod parser;
mod type_registry;

use generator::{
    code_generator_trait::{CodeGenOptions, CodeGenerator},
    delphi::code_generator::DelphiCodeGenerator,
    internal_representation::InternalRepresentation,
};
use parser::{types::ParsedData, xml::XmlParser};
use type_registry::TypeRegistry;

fn main() {
    let args = Args::parse();
    let instant = Instant::now();

    let output_path = match resolve_output_path(&args.output) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{}", e);

            return;
        }
    };

    let output_file = match File::create(output_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!(
                "Could not create output file due to following error: \"{:?}\"",
                e
            );
            return;
        }
    };

    let mut parser = XmlParser::default();
    let mut type_registry = TypeRegistry::new();

    let data: ParsedData = if args.input.len() == 1 {
        match parser.parse_file(args.input.first().unwrap(), &mut type_registry) {
            Ok(n) => n,
            Err(error) => {
                eprintln!("An error occured: {}", error);
                return;
            }
        }
    } else {
        match parser.parse_files(&args.input, &mut type_registry) {
            Ok(n) => n,
            Err(error) => {
                eprintln!("An error occured: {}", error);
                return;
            }
        }
    };

    let elapsed_for_parse = instant.elapsed().as_millis();
    println!("Files parsed in {}ms", elapsed_for_parse);

    let internal_representation = InternalRepresentation::build(&data, &type_registry);

    let elapsed_for_ir = instant
        .elapsed()
        .as_millis()
        .saturating_sub(elapsed_for_parse);
    println!("Internal Representation created in {}ms", elapsed_for_ir);

    let buffer = BufWriter::new(Box::new(output_file));
    let mut generator = DelphiCodeGenerator::new(
        buffer,
        build_code_gen_options(&args),
        internal_representation,
        data.documentations,
    );

    match generator.generate() {
        Ok(_) => println!(
            "Completed successfully within {}ms",
            instant.elapsed().as_millis().saturating_sub(elapsed_for_ir),
        ),
        Err(e) => {
            eprintln!(
                "Failed to write output to file due to following error: \"{:?}\"",
                e
            );
        }
    }
}

fn build_code_gen_options(args: &Args) -> CodeGenOptions {
    CodeGenOptions {
        generate_from_xml: !matches!(&args.mode, CodeGenMode::ToXml),
        generate_to_xml: !matches!(&args.mode, CodeGenMode::FromXml),
        unit_name: args.unit_name.clone(),
        type_prefix: args.type_prefix.clone(),
    }
}

fn resolve_output_path(path: &PathBuf) -> Result<PathBuf, String> {
    if path.is_relative() {
        std::env::current_dir().map(|d| d.join(path)).map_err(|e| {
            format!(
                "Relative path not supported due to following error: \"{:?}\"",
                e
            )
        })
    } else {
        path.canonicalize().map_err(|e| {
            format!(
                "Could not resolve output path due to following error: \"{:?}\"",
                e
            )
        })
    }
}

/// XSD2DelphiCodeGen generates Types from XSD-Files
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// One or multiple paths to xsd files. Paths can be relative or absolut.
    #[arg(short, long, value_hint = clap::ValueHint::DirPath, num_args(1..))]
    pub(crate) input: Vec<std::path::PathBuf>,

    /// Path to output file. Path can be relative or absolut. File will be created or truncated before write.
    #[arg(short, long, required(true))]
    pub(crate) output: std::path::PathBuf,

    #[arg(long, required(true))]
    pub(crate) unit_name: String,

    /// Optional prefix for type names
    #[arg(long, num_args(0..=1))]
    pub(crate) type_prefix: Option<String>,

    #[arg(long, value_enum, default_value_t)]
    pub(crate) mode: CodeGenMode,
}

#[derive(Clone, Debug, Default, ValueEnum)]
enum CodeGenMode {
    #[default]
    All,
    ToXml,
    FromXml,
}
