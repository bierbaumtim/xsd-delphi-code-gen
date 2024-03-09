use std::path::PathBuf;

use serde::Serialize;
use sw4rm_rs::{from_path, shared::SchemaType};
use tera::{Context, Tera};

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

    // TODO: Add TypeRegistry
    // TODO: Iterate over all schemas and register classes and enums
    // TODO: Iterate over all paths and register classes and enums
    // TODO: Iterate over all paths and generate endpoints
    // TODO: Iterate over all types in the TypeRegistry and generate classes and enums
    // TODO: Build context for client template
    // TODO: Build context for client interface template
    // TODO: Build context for models template

    let class_types = vec![ClassType {
        name: "Test".to_string(),
        needs_destructor: true,
        properties: vec![
            Property {
                name: "p1".to_string(),
                type_name: "string".to_string(),
                key: "p1".to_string(),
                is_reference_type: false,
                is_list_type: false,
                is_enum_type: false,
            },
            Property {
                name: "p2".to_string(),
                type_name: "integer".to_string(),
                key: "p2".to_string(),
                is_reference_type: true,
                is_list_type: false,
                is_enum_type: false,
            },
            Property {
                name: "p3".to_string(),
                type_name: "integer".to_string(),
                key: "p3".to_string(),
                is_reference_type: true,
                is_list_type: true,
                is_enum_type: false,
            },
            Property {
                name: "p4".to_string(),
                type_name: "integer".to_string(),
                key: "p4".to_string(),
                is_reference_type: false,
                is_list_type: true,
                is_enum_type: false,
            },
            Property {
                name: "p5".to_string(),
                type_name: "Test".to_string(),
                key: "p5".to_string(),
                is_reference_type: false,
                is_list_type: true,
                is_enum_type: true,
            },
            Property {
                name: "p6".to_string(),
                type_name: "Test".to_string(),
                key: "p6".to_string(),
                is_reference_type: false,
                is_list_type: false,
                is_enum_type: true,
            },
        ],
    }];

    let class_types = openapi_spec
        .schemas()
        .iter()
        .filter_map(|(k, v)| {
            v.resolve(&openapi_spec).ok().and_then(|s| {
                if !s.schema_type.is_some_and(|t| t == SchemaType::Object) {
                    return None;
                }

                let properties = s
                    .properties
                    .iter()
                    .filter_map(|(k, v)| {
                        v.resolve(&openapi_spec).ok().map(|s| Property {
                            name: capitalize(k),
                            type_name: s.schema_type.as_ref().map_or("".to_string(), |t| match t {
                                SchemaType::String => match &s.format {
                                    Some(f) => match f.as_str() {
                                        "date" | "date-time" => "datetime".to_string(),
                                        _ => "string".to_string(),
                                    },
                                    None => "string".to_string(),
                                },
                                SchemaType::Integer => "integer".to_string(),
                                SchemaType::Number => "double".to_string(),
                                SchemaType::Boolean => "boolean".to_string(),
                                SchemaType::Array => s
                                    .items
                                    .as_ref()
                                    .expect("Array must have items property set")
                                    .resolve(&openapi_spec)
                                    .expect("Type of array items must be resolved")
                                    .schema_type
                                    .as_ref()
                                    .map_or("".to_string(), |t| match t {
                                        SchemaType::String => match &s.format {
                                            Some(f) => match f.as_str() {
                                                "date" | "date-time" => "datetime".to_string(),
                                                _ => "string".to_string(),
                                            },
                                            None => "string".to_string(),
                                        },
                                        SchemaType::Integer => "integer".to_string(),
                                        SchemaType::Number => "double".to_string(),
                                        SchemaType::Boolean => "boolean".to_string(),
                                        _ => "".to_string(),
                                    }),
                                _ => s.title.clone().unwrap_or(k.to_string()),
                            }),
                            key: k.to_owned(),
                            is_reference_type: s
                                .schema_type
                                .is_some_and(|t| t == SchemaType::Object)
                                || s.items.is_some_and(|i| {
                                    i.resolve(&openapi_spec).ok().is_some_and(|s| {
                                        s.schema_type.is_some_and(|t| {
                                            matches!(t, SchemaType::Object | SchemaType::Array)
                                        })
                                    })
                                }),
                            is_list_type: s.schema_type.is_some_and(|t| t == SchemaType::Array),
                            is_enum_type: !s.enum_values.is_empty(),
                        })
                    })
                    .collect::<Vec<Property>>();

                Some(ClassType {
                    name: capitalize(k),
                    needs_destructor: properties.iter().any(|p| p.is_reference_type),
                    properties,
                })
            })
        })
        .collect::<Vec<ClassType>>();

    let enum_types = openapi_spec
        .schemas()
        .iter()
        .filter_map(|(k, v)| {
            v.resolve(&openapi_spec).ok().map(|s| {
                let variants = s
                    .enum_values
                    .iter()
                    .filter_map(|v| {
                        v.as_str().map(|s| EnumVariant {
                            name: capitalize(s),
                            key: s.to_owned(),
                        })
                    })
                    .collect();

                EnumType {
                    name: capitalize(k),
                    variants,
                }
            })
        })
        .collect::<Vec<EnumType>>();

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

fn capitalize(value: &str) -> String {
    let mut c = value.chars();

    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

#[derive(Serialize)]
struct ClassType {
    name: String,
    properties: Vec<Property>,
    needs_destructor: bool,
}

#[derive(Serialize)]
struct Property {
    name: String,
    type_name: String,
    key: String,
    is_reference_type: bool,
    is_list_type: bool,
    is_enum_type: bool,
}

#[derive(Serialize)]
struct EnumType {
    name: String,
    variants: Vec<EnumVariant>,
}

#[derive(Serialize)]
struct EnumVariant {
    name: String,
    key: String,
}
