use std::{fs::File, io::Write, path::PathBuf};

use clap::Parser;

mod generator;
mod parser;
mod parser_types;
mod type_registry;

use parser::Parser as XmlParser;
use parser_types::Node;
use type_registry::TypeRegistry;

fn main() {
    let args = Args::parse();

    let output_path: PathBuf;
    if args.output.is_relative() {
        let dir = match std::env::current_dir() {
            Ok(d) => d,
            Err(e) => {
                println!(
                    "Relative path not supported due to following error: \"{:?}\"",
                    e
                );
                return;
            }
        };

        output_path = dir.join(args.output);
    } else {
        output_path = match args.output.canonicalize() {
            Ok(p) => p,
            Err(e) => {
                println!(
                    "Could not resolve output path due to following error: \"{:?}\"",
                    e
                );
                return;
            }
        };
    }

    let mut output_file = match File::create(output_path) {
        Ok(f) => f,
        Err(e) => {
            println!(
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
                println!("An error occured: {}", error);
                return;
            }
        }
    } else {
        match parser.parse_files(args.input, &mut type_registry) {
            Ok(n) => n,
            Err(error) => {
                println!("An error occured: {}", error);
                return;
            }
        }
    };

    // println!("Nodes: {:#?}", nodes);
    // println!("");
    // println!("Types: {:#?}", type_registry.types);

    let res = output_file
        .write_all(format!("Nodes: {:#?}\n\nTypes: {:#?}", nodes, type_registry.types).as_bytes());

    match res {
        Ok(_) => println!("Completed successfully"),
        Err(e) => {
            println!(
                "Failed to write output to file due to following error: \"{:?}\"",
                e
            );
            return;
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
    pub(crate) unit_name: Option<String>,

    #[arg(long, num_args(0..=1))]
    pub(crate) type_prefix: Option<String>,
}
