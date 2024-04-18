use std::path::{Path, PathBuf};

use sw4rm_rs::from_path;
use tera::Tera;

mod endpoint_collector;
mod helper;
mod models;
mod render;
mod schema_collector;
mod type_registry;

pub fn generate_openapi_client(source: &[PathBuf], dest: &Path, prefix: Option<String>) {
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

    let macros_template_str = include_str!("templates/macros.pas");
    let client_template_str = include_str!("templates/client.pas");
    let client_interface_template_str = include_str!("templates/client_interface.pas");
    let models_template_str = include_str!("templates/models.pas");

    let mut tera = Tera::default();
    if let Err(e) = tera.add_raw_template("macros.pas", macros_template_str) {
        eprintln!("Failed to add macros template due to {:?}", e);

        return;
    }
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
    // TODO: Build context for client template

    let (mut class_types, mut enum_types) =
        schema_collector::collect_types(&openapi_spec, prefix.clone());
    let endpoints =
        endpoint_collector::collect_endpoints(&openapi_spec, &mut class_types, &mut enum_types);

    render::render_models(
        &openapi_spec,
        dest,
        prefix.clone(),
        &class_types,
        &enum_types,
        &tera,
    );
    render::render_client_interface(&openapi_spec, dest, prefix.clone(), &endpoints, &tera);
    render::render_client(&openapi_spec, dest, prefix.clone(), &endpoints, &tera);
}
