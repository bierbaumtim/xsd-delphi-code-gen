use clap::Parser;

mod generator;
mod parser;
mod parser_types;
mod type_registry;

use parser::Parser as XmlParser;
use type_registry::TypeRegistry;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, value_hint = clap::ValueHint::DirPath)]
    input: std::path::PathBuf,

    #[arg(short, long)]
    output: std::path::PathBuf,

    #[arg(long)]
    unit_name: Option<String>,

    #[arg(long)]
    type_prefix: Option<String>,
}

fn main() {
    let args = Args::parse();

    let mut parser = XmlParser::default();
    let mut type_registry = TypeRegistry::new();

    let nodes = match parser.parse_file(args.input, &mut type_registry) {
        Ok(n) => n,
        Err(error) => {
            println!("An error occured: {}", error);
            return;
        }
    };

    println!("Nodes: {:#?}", nodes);
    println!("");
    println!("Types: {:#?}", type_registry.types);
}
