use sw4rm_rs::Spec;
use tera::{Context, Tera};

use crate::models::{ClassType, Endpoint, EnumType};

pub(crate) fn render_models(
    spec: &Spec,
    dest: &std::path::PathBuf,
    prefix: Option<String>,
    class_types: &[ClassType],
    enum_types: &[EnumType],
    tera: &Tera,
) {
    let mut models_context = Context::new();
    models_context.insert("unitPrefix", &prefix.clone().unwrap_or_default());
    models_context.insert("prefix", &prefix.clone().unwrap_or_default());
    models_context.insert("crate_version", "0.0.1");
    models_context.insert("api_title", &spec.info.title);
    models_context.insert("api_spec_version", &spec.info.version);
    models_context.insert("classTypes", &class_types);
    models_context.insert("enumTypes", &enum_types);

    let models = tera.render("models.pas", &models_context);

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

pub(crate) fn render_client_interface(
    spec: &Spec,
    dest: &std::path::PathBuf,
    prefix: Option<String>,
    endpoints: &[Endpoint],
    tera: &Tera,
) {
    let mut models_context = Context::new();
    models_context.insert("unitPrefix", &prefix.clone().unwrap_or_default());
    models_context.insert("prefix", &prefix.clone().unwrap_or_default());
    models_context.insert("crate_version", "0.0.1");
    models_context.insert("api_title", &spec.info.title);
    models_context.insert("api_spec_version", &spec.info.version);
    models_context.insert("endpoints", &endpoints);

    let models = tera.render("client_interface.pas", &models_context);

    match models {
        Ok(s) => {
            let models_path = dest.join(format!(
                "u{}ApiClientInterface.pas",
                prefix.unwrap_or_default()
            ));
            if let Err(e) = std::fs::write(models_path, s) {
                eprintln!("Failed to write client interface file due to {:?}", e);
            }
        }
        Err(e) => eprintln!("Failed to render client interface template due to {:?}", e),
    }
}

pub(crate) fn render_client(
    spec: &Spec,
    dest: &std::path::PathBuf,
    prefix: Option<String>,
    endpoints: &[Endpoint],
    tera: &Tera,
) {
    let mut models_context = Context::new();
    models_context.insert("unitPrefix", &prefix.clone().unwrap_or_default());
    models_context.insert("prefix", &prefix.clone().unwrap_or_default());
    models_context.insert("crate_version", "0.0.1");
    models_context.insert("api_title", &spec.info.title);
    models_context.insert("api_spec_version", &spec.info.version);
    models_context.insert("endpoints", &endpoints);

    let models = tera.render("client.pas", &models_context);

    match models {
        Ok(s) => {
            let models_path = dest.join(format!("u{}ApiClient.pas", prefix.unwrap_or_default()));
            if let Err(e) = std::fs::write(models_path, s) {
                eprintln!("Failed to write client file due to {:?}", e);
            }
        }
        Err(e) => eprintln!("Failed to render client template due to {:?}", e),
    }
}
