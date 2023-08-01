use std::{
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
    time::Instant,
};

use clap::{Parser, ValueEnum};

mod generator;
mod parser;
mod parser_types;
mod type_registry;

use generator::{
    code_generator_trait::{CodeGenOptions, CodeGenerator},
    delphi::code_generator::DelphiCodeGenerator,
    internal_representation::InternalRepresentation,
};
use parser::Parser as XmlParser;
use parser_types::Node;
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

    let nodes: Vec<Node> = if args.input.len() == 1 {
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

    let internal_representation = InternalRepresentation::build(&nodes, &type_registry);
    let mut buffer: BufWriter<Box<dyn Write>> = BufWriter::new(Box::new(output_file));
    let mut generator = DelphiCodeGenerator::new(
        &mut buffer,
        build_code_gen_options(&args),
        internal_representation,
    );
    
    match generator.generate() {
        Ok(_) => println!(
            "Completed successfully within {}ms",
            instant.elapsed().as_millis()
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
    }
}

fn resolve_output_path(path: &PathBuf) -> Result<PathBuf, String> {
    if path.is_relative() {
        let dir = match std::env::current_dir() {
            Ok(d) => d,
            Err(e) => {
                return Err(format!(
                    "Relative path not supported due to following error: \"{:?}\"",
                    e
                ));
            }
        };

        Ok(dir.join(path))
    } else {
        match path.canonicalize() {
            Ok(p) => Ok(p),
            Err(e) => Err(format!(
                "Could not resolve output path due to following error: \"{:?}\"",
                e
            )),
        }
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(short, long, value_hint = clap::ValueHint::DirPath, num_args(1..))]
    pub(crate) input: Vec<std::path::PathBuf>,

    #[arg(short, long, required(true))]
    pub(crate) output: std::path::PathBuf,

    #[arg(long, required(true))]
    pub(crate) unit_name: String,

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
