use anyhow::Ok;

use crate::parser::types::*;

use genphi_core::ir::{
    type_id_provider::UNRESOLVED_TYPE_ID, type_registry::TypeRegistry, types::*,
};

// TODO: Odir-like type registry as central place for all types
// TODO: Resolve by name (ex. Schema name -> is unique by design through OpenAPI spec) -> internal name = "schema_{schema.name}"
// TODO: IrTypeId = usize => easy referencing and simple check for same type for endpoints responses -> same type differnt serialization

pub fn generate_code(spec: &OpenAPI) -> anyhow::Result<(String, String)> {
    let mut models_registry = TypeRegistry::new();

    let models_unit = generate_models_unit(spec, &mut models_registry);

    let mut client_unit = DelphiUnit::default();
    client_unit.unit_name = build_unit_name(spec, "Client");

    let mut client_class = DelphiClass::new(
        format!("TM{}", spec.info.title.replace(['-', '.', ':', ' '], "")),
        "__internal__",
    );

    for (name, path) in spec.paths.iter() {
        if let Some(get) = &path.get {
            for (r_name, response) in get.responses.iter() {
                let response = match response {
                    ResponseOrRef::Item(response) => Some(response),
                    ResponseOrRef::Ref { reference } => spec.resolve_response(reference),
                };

                let Some(response) = response else {
                    continue;
                };

                for (m_name, media_type) in response.content.iter() {
                    let (schema, reference) = match media_type.schema.as_ref() {
                        Some(SchemaOrRef::Item(schema)) => (Some(schema), None),
                        Some(SchemaOrRef::Ref { reference }) => {
                            (spec.resolve_schema(reference), Some(reference))
                        }
                        None => (None, None),
                    };

                    let Some(schema) = schema else {
                        continue;
                    };

                    let delphi_type = register_schema(spec, &mut models_registry, schema, None);
                }
            }

            // let method_name = build_operation_method_name(&get.operation_id, name);

            // let method = DelphiMethod {
            //     name: method_name,
            //     visibility: DelphiVisibility::Public,
            //     return_type: DelphiType::Void,
            //     parameters: vec![],
            // };
        }
    }

    client_unit.classes.push(client_class);

    Ok((String::new(), String::new()))
}

fn generate_models_unit(spec: &OpenAPI, models_registry: &mut TypeRegistry) -> DelphiUnit {
    let mut unit = DelphiUnit::default();
    unit.unit_name = build_unit_name(spec, "Models");

    let mut class = DelphiClass::new(
        format!(
            "TM{}Models",
            spec.info.title.replace(['-', '.', ':', ' '], "")
        ),
        "",
    );

    let Some(components) = &spec.components else {
        return unit;
    };

    for (name, schema) in components.schemas.iter() {
        let schema = match schema {
            SchemaOrRef::Item(schema) => Some(schema),
            SchemaOrRef::Ref { reference } => spec.resolve_schema(reference),
        };

        let Some(schema) = schema else {
            continue;
        };

        let r#type = register_schema(spec, models_registry, schema, Some(name.clone()));
    }

    // for (name, body) in components.request_bodies.iter() {
    //     let body = match body {
    //         RequestBodyOrRef::Item(request_body) => Some(request_body),
    //         RequestBodyOrRef::Ref { reference } => spec.resolve_request_body(reference),
    //     };

    //     let Some(body) = body else {
    //         continue;
    //     };

    //     let base_class = if body.content.len() > 1 {
    //         let mut temp = DelphiClass::new("TM{name}Base");
    //         temp.methods.push(DelphiMethod {
    //             comment: None,
    //             name: "AsString".to_owned(),
    //             is_class_method: false,
    //             is_constructor: false,
    //             is_destructor: false,
    //             is_override: false,
    //             is_static: false,
    //             is_virtual: false,
    //             parameters: vec![],
    //             visibility: DelphiVisibility::Public,
    //             return_type: Some("String".to_owned()),
    //         });

    //         Some(temp)
    //     } else {
    //         None
    //     };

    //     for (c_name, content) in body.content.iter() {
    //         let Some(schema) = &content.schema else {
    //             continue;
    //         };

    //         let (schema, reference) = match schema {
    //             SchemaOrRef::Item(schema) => Some(schema),
    //             SchemaOrRef::Ref { reference } => spec.resolve_schema(reference),
    //         };

    //         let Some(schema) = schema else {
    //             continue;
    //         };

    //     }
    // }

    unit
}

fn register_schema(
    spec: &OpenAPI,
    models_registry: &mut TypeRegistry,
    schema: &Schema,
    class_name: Option<String>,
) -> Option<DelphiType> {
    let Some(r#type) = schema.r#type.as_ref() else {
        return None;
    };

    return match r#type.as_str() {
        "string" => {
            if schema.enum_.is_empty() {
                Some(DelphiType::String)
            } else {
                let (gen_id, name) = class_name.or(schema.title.clone()).map_or_else(
                    || TypeRegistry::generate_name("Enum"),
                    |n| (UNRESOLVED_TYPE_ID, n),
                );

                let name = format!("T{}", name);
                let id = models_registry.find_enum_type_id_by_name(&name, "schema");
                if let Some(id) = id {
                    return Some(DelphiType::Enum(id));
                }

                let variants = schema
                    .enum_
                    .iter()
                    .map(|v| {
                        let name = v.to_string();

                        DelphiEnumVariant {
                            name: name.replace(['-', '.', ':', ' ', '/', '\\'], ""),
                            value: Some(name),
                            comment: None,
                        }
                    })
                    .collect::<Vec<_>>();

                let internal_name = TypeRegistry::build_internal_name(&name, "schema");
                let enum_ = DelphiEnum {
                    name,
                    internal_name,
                    variants,
                    comment: None,
                };

                let id = if gen_id == UNRESOLVED_TYPE_ID {
                    models_registry.register_enum(enum_)
                } else {
                    models_registry.register_enum_with_id(gen_id, enum_);

                    gen_id
                };

                Some(DelphiType::Enum(id))
            }
        }
        "integer" => Some(DelphiType::Integer),
        "number" => match &schema.format {
            Some(format) if format == "float" => Some(DelphiType::Float),
            Some(format) if format == "double" => Some(DelphiType::Double),
            _ => Some(DelphiType::Double),
        },
        "boolean" => Some(DelphiType::Boolean),
        "array" => {
            if let Some(items) = &schema.items {
                let items: &SchemaOrRef = items;

                let schema = match items {
                    SchemaOrRef::Item(s) => s,
                    SchemaOrRef::Ref { reference } => spec.resolve_schema(reference)?,
                };

                let inner_type = register_schema(
                    spec,
                    models_registry,
                    schema,
                    schema.title.as_ref().cloned(),
                )?;

                Some(DelphiType::List(Box::new(inner_type)))
            } else {
                Some(DelphiType::List(Box::new(DelphiType::Pointer)))
            }
        }
        "object" => {
            // Handle object type
            let (gen_id, name) = class_name.or(schema.title.clone()).map_or_else(
                || TypeRegistry::generate_name("Enum"),
                |n| (UNRESOLVED_TYPE_ID, n),
            );

            let name = format!("T{}", name);
            let id = models_registry.find_class_type_id_by_name(&name, "schema");
            if let Some(id) = id {
                return Some(DelphiType::Class(id));
            }

            let mut class = DelphiClass::new(name, "schema");

            // Process properties
            for (prop_name, prop) in schema.properties.iter() {
                let prop_type = match &prop {
                    SchemaOrRef::Item(s) => Some(s),
                    SchemaOrRef::Ref { reference } => spec.resolve_schema(reference),
                };

                let Some(prop) = prop_type else {
                    continue;
                };

                let delphi_type =
                    register_schema(spec, models_registry, prop, Some(prop_name.clone()));

                if let Some(delphi_type) = delphi_type {
                    class.fields.push(DelphiField {
                        name: prop_name.clone(),
                        is_reference_type: matches!(
                            delphi_type,
                            DelphiType::List(_) | DelphiType::Class(_)
                        ),
                        field_type: delphi_type,
                        visibility: DelphiVisibility::Public,
                        json_key: Some(prop_name.clone()),
                        xml_attribute: None,
                        comment: None,
                        default_value: None,
                    });
                }
            }

            let id = if gen_id == UNRESOLVED_TYPE_ID {
                models_registry.register_class(class)
            } else {
                models_registry.register_class_with_id(gen_id, class);

                gen_id
            };

            Some(DelphiType::Class(id))
        }
        _ => None,
    };

    // let mut class = DelphiClass::new(class_name, "");

    // // Process properties
    // for (name, prop) in schema.properties.iter() {
    //     let prop_type = match &prop {
    //         SchemaOrRef::Item(s) => Some(s),
    //         SchemaOrRef::Ref { reference } => spec.resolve_schema(reference),
    //     };

    //     let Some(prop) = prop_type else {
    //         continue;
    //     };

    //     let delphi_type = data_type_from_schema(spec, prop, name);
    //     let Some(delphi_type) = delphi_type else {
    //         continue;
    //     };

    //     class.fields.push(DelphiField {
    //         name: name.clone(),
    //         field_type: delphi_type,
    //         visibility: DelphiVisibility::Public,
    //         is_reference_type: false,
    //         json_key: None,
    //         xml_attribute: None,
    //         comment: None,
    //         default_value: None,
    //     });
    // }

    // Add other properties like methods, constructors, etc. as needed

    // Some(class)
}

fn build_unit_name(spec: &OpenAPI, trailing: &str) -> String {
    let title = spec.info.title.replace(['-', '.', ':', ' '], "");

    format!("uM{title}{trailing}")
}

fn build_operation_method_name(operation_id: &Option<String>, path: &str) -> String {
    if let Some(id) = operation_id {
        id.replace(['-', '.', ':', ' ', '/', '\\'], "")
    } else {
        let path_part = path
            .split('/')
            .filter(|p| !p.is_empty())
            .last()
            .unwrap_or("unknown");

        format!("Get{path_part}")
    }
}
