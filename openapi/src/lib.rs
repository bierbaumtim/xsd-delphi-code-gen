use std::path::PathBuf;

use sw4rm_rs::from_path;
use tera::{Context, Tera};

mod helper;
mod models;
mod schema_collector;
mod type_registry;

pub fn generate_openapi_client(source: &[PathBuf], dest: &PathBuf, prefix: Option<String>) {
    let source = match source.first() {
        Some(p) => p,
        None => {
            eprintln!("No source file provided");

            return;
        }
    };

    if !dest.is_dir() {
        eprintln!("Destination path is not a directory");

        return;
    }

    let openapi_spec = match from_path(source) {
        Ok(spec) => spec,
        Err(e) => {
            eprintln!(
                "Failed to parse OpenAPI Spec file at {:?} due to {:?}",
                source, e
            );

            return;
        }
    };

    let client_template_str = include_str!("templates/client.pas");
    let client_interface_template_str = include_str!("templates/client_interface.pas");
    let models_template_str = include_str!("templates/models.pas");

    let mut tera = Tera::default();
    if let Err(e) = tera.add_raw_template("client.pas", client_template_str) {
        eprintln!("Failed to add client template due to {:?}", e);

        return;
    }
    if let Err(e) = tera.add_raw_template("client_interface.pas", client_interface_template_str) {
        eprintln!("Failed to add client interface template due to {:?}", e);

        return;
    }
    if let Err(e) = tera.add_raw_template("models.pas", models_template_str) {
        eprintln!("Failed to add models template due to {:?}", e);

        return;
    }

    // TODO: Iterate over all paths and generate endpoints
    // TODO: Iterate over all types in the TypeRegistry and generate classes and enums
    // TODO: Build context for client template
    // TODO: Build context for client interface template
    // TODO: Build context for models template

    let (class_types, enum_types) = schema_collector::collect_types(&openapi_spec, prefix.clone());

    let mut context = Context::new();
    context.insert("unitPrefix", &prefix.clone().unwrap_or_default());
    context.insert("prefix", &prefix.clone().unwrap_or_default());
    context.insert("crate_version", "0.0.1");
    context.insert("api_title", &openapi_spec.info.title);
    context.insert("api_spec_version", &openapi_spec.info.version);
    context.insert("classTypes", &class_types);
    context.insert("enumTypes", &enum_types);

    let models = tera.render("models.pas", &context);

    match models {
        Ok(s) => {
            let models_path = dest.join(format!("u{}ApiModels.pas", prefix.unwrap_or_default()));
            if let Err(e) = std::fs::write(models_path, s) {
                eprintln!("Failed to write models file due to {:?}", e);
            }
        }
        Err(e) => eprintln!("Failed to render model template due to {:?}", e),
    }
}
