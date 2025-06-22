#![allow(clippy::too_many_lines)]
use std::path::PathBuf;

use clap::{Parser, ValueEnum};

use openapi::{generate_openapi_client, start_spec_browser};
use xml::{generate_xml, generator::code_generator_trait::CodeGenOptions};

fn main() {
    let args = Args::parse();

    let output_path = match resolve_output_path(&args.output) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{e}");

            return;
        }
    };

    match &args.source_format {
        SourceFormat::Xml => generate_xml(&args.input, &output_path, build_code_gen_options(&args)),
        SourceFormat::OpenApi => {
            if args.input.is_empty() {
                eprintln!("No input files provided for OpenAPI generation");

                return;
            }

            let _ = start_spec_browser(args.input.first().expect("").clone());
            // generate_openapi_client(&args.input, &output_path, &args.type_prefix)
        }
    }
}

fn build_code_gen_options(args: &Args) -> CodeGenOptions {
    CodeGenOptions {
        generate_from_xml: !matches!(&args.mode, CodeGenMode::ToXml),
        generate_to_xml: !matches!(&args.mode, CodeGenMode::FromXml),
        unit_name: args.unit_name.clone().expect("Unit name is required"),
        type_prefix: args.type_prefix.clone(),
    }
}

fn resolve_output_path(path: &PathBuf) -> Result<PathBuf, String> {
    if path.is_relative() {
        std::env::current_dir()
            .map(|d| d.join(path))
            .map_err(|e| format!("Relative path not supported due to following error: \"{e:?}\""))
    } else {
        std::path::absolute(path)
            .map_err(|e| format!("Could not resolve output path due to following error: \"{e:?}\""))
    }
}

/// `XSD2DelphiCodeGen` generates Types from XSD-Files for Delphi
/// # Usage
///
/// ```bash
/// XSD2DelphiCodeGen [FLAGS] [OPTIONS] <input> <output> <unit-name>
/// ```
///
/// # Arguments
///
/// * `<input>` - One or multiple paths to xsd files. Paths can be relative or absolut.
/// * `<output>` - Path to output file. Path can be relative or absolut. File will be created or truncated before write.
/// * `<unit-name>` - Name of the generated unit
///
/// # Options
///
/// * `--mode <mode>` - Which code should be generated. Can be one of `All`, `ToXml`, `FromXml`. Default is `All`
/// * `--type-prefix <type-prefix>` - Optional prefix for type names
///
/// # Flags
///
/// * `-h, --help` - Prints help information
/// * `-V, --version` - Prints version information
///
/// # Examples
///
/// ```bash
/// # Generate all code
/// XSD2DelphiCodeGen input.xsd output.pas MyUnit
///
/// # Generate only code for xml to type conversion
/// XSD2DelphiCodeGen --mode ToXml input.xsd output.pas MyUnit
///
/// # Generate only code for type to xml conversion
/// XSD2DelphiCodeGen --mode FromXml input.xsd output.pas MyUnit
///
/// # Generate code with prefix
/// XSD2DelphiCodeGen --type-prefix MyPrefix input.xsd output.pas MyUnit
/// ```
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// One or multiple paths to xsd files. Paths can be relative or absolut.
    #[arg(short, long, value_hint = clap::ValueHint::DirPath, num_args(1..))]
    pub(crate) input: Vec<std::path::PathBuf>,

    /// Path to output file. Path can be relative or absolut. File will be created or truncated before write.
    #[arg(short, long, required(true))]
    pub(crate) output: std::path::PathBuf,

    /// Name of the generated unit
    #[arg(long)]
    pub(crate) unit_name: Option<String>,

    /// Optional prefix for type names
    #[arg(long, num_args(0..=1))]
    pub(crate) type_prefix: Option<String>,

    /// Which code should be generated. Can be one of `All`, `ToXml`, `FromXml`. Default is `All`
    #[arg(long, value_enum, default_value_t)]
    pub(crate) mode: CodeGenMode,

    /// Source format of the input files. Can be one of `Xml`, `OpenApi`. Default is `Xml`
    #[arg(long, value_enum)]
    pub(crate) source_format: SourceFormat,
}

/// Which code should be generated. Can be one of `All`, `ToXml`, `FromXml`. Default is `All`
#[derive(Clone, Debug, Default, ValueEnum)]
enum CodeGenMode {
    /// Generate all code
    #[default]
    All,

    /// Generate only code for type to xml conversion
    ToXml,

    /// Generate only code for xml to type conversion
    FromXml,
}

/// Source format of the input files. Can be one of `Xml`, `OpenApi`. Default is `Xml`
#[derive(Clone, Debug, ValueEnum)]
enum SourceFormat {
    Xml,
    OpenApi,
}
