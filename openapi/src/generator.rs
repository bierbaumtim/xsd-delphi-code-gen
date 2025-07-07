use anyhow::Ok;

use crate::parser::types::*;

use codegen::*;
use genphi_core::ir::{
    IrTypeIdOrName, type_id_provider::UNRESOLVED_TYPE_ID, type_registry::TypeRegistry, types::*,
};

// TODO: Odir-like type registry as central place for all types
// TODO: Resolve by name (ex. Schema name -> is unique by design through OpenAPI spec) -> internal name = "schema_{schema.name}"
// TODO: IrTypeId = usize => easy referencing and simple check for same type for endpoints responses -> same type differnt serialization

pub fn generate_code(spec: &OpenAPI) -> anyhow::Result<(String, String)> {
    let mut models_registry = TypeRegistry::new();

    let mut client_unit = DelphiUnit::default();
    client_unit.unit_name = build_unit_name(spec, "Client");

    let client_class = DelphiClass::new(
        format!("TM{}", spec.info.title.replace(['-', '.', ':', ' '], "")),
        "__internal__",
    );

    for (_, path) in spec.paths.iter() {
        if let Some(get) = &path.get {
            register_operation_schema(spec, &mut models_registry, get);
        }

        if let Some(post) = &path.post {
            register_operation_schema(spec, &mut models_registry, post);
        }

        if let Some(put) = &path.put {
            register_operation_schema(spec, &mut models_registry, put);
        }

        if let Some(delete) = &path.delete {
            register_operation_schema(spec, &mut models_registry, delete);
        }

        if let Some(patch) = &path.patch {
            register_operation_schema(spec, &mut models_registry, patch);
        }

        if let Some(head) = &path.head {
            register_operation_schema(spec, &mut models_registry, head);
        }

        if let Some(options) = &path.options {
            register_operation_schema(spec, &mut models_registry, options);
        }

        if let Some(trace) = &path.trace {
            register_operation_schema(spec, &mut models_registry, trace);
        }
    }

    client_unit.classes.push(client_class);

    let models_unit = generate_models_unit(spec, &mut models_registry);
    let cg = DelphiCodeGenerator::new(CodeGenConfig::default());
    let models_code = cg.generate_unit(&models_unit)?;

    Ok((String::new(), models_code))
}

fn register_operation_schema(
    spec: &OpenAPI,
    models_registry: &mut TypeRegistry,
    operation: &Operation,
) {
    for (r_name, response) in operation.responses.iter() {
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
                Some(SchemaOrRef::Ref { reference }) => (
                    spec.resolve_schema(reference),
                    reference.split("/").last().map(|v| v.to_owned()),
                ),
                None => (None, None),
            };

            let Some(schema) = schema else {
                continue;
            };

            let _ = register_schema(spec, models_registry, schema, reference, &m_name);
        }
    }
}

fn generate_models_unit(spec: &OpenAPI, models_registry: &mut TypeRegistry) -> DelphiUnit {
    let mut unit = DelphiUnit::default();
    unit.unit_name = build_unit_name(spec, "Models");

    for enum_type in models_registry.enum_iter() {
        let enum_class = DelphiEnum {
            name: enum_type.name.clone(),
            internal_name: enum_type.internal_name.clone(),
            comment: enum_type.comment.clone(),
            variants: enum_type
                .variants
                .iter()
                .map(|v| DelphiEnumVariant {
                    name: v.name.clone(),
                    value: v.value.clone(),
                    comment: v.comment.clone(),
                })
                .collect(),
        };

        unit.enums.push(enum_class);
    }

    for class_type in models_registry.classes_iter() {
        let class = DelphiClass {
            name: class_type.name.clone(),
            internal_name: class_type.internal_name.clone(),
            comment: class_type.comment.clone(),
            parent_class: None, // TODO:
            generate_json_support: true,
            generate_xml_support: false,
            fields: class_type
                .fields
                .iter()
                .map(|f| DelphiField {
                    name: capitalize(&f.name),
                    field_type: f.field_type.resolve(&models_registry),
                    comment: f.comment.clone(),
                    visibility: DelphiVisibility::StrictPrivate,
                    is_reference_type: matches!(
                        f.field_type,
                        DelphiType::Class(_) | DelphiType::List(_)
                    ),
                    json_key: f.json_key.clone(),
                    is_required: f.is_required,
                    default_value: f.default_value.clone(),
                    xml_info: f.xml_info.clone(),
                })
                .collect(),
            properties: class_type
                .fields
                .iter()
                .map(|f| DelphiProperty {
                    name: capitalize(&f.name),
                    property_type: f.field_type.resolve(&models_registry),
                    getter: Some(capitalize(&f.name)),
                    setter: if !matches!(f.field_type, DelphiType::Class(_) | DelphiType::List(_)) {
                        Some(capitalize(&f.name))
                    } else {
                        None
                    },
                    visibility: DelphiVisibility::Public,
                    comment: f.comment.clone(),
                })
                .collect(),
            methods: vec![],
        };

        unit.classes.push(class);
    }

    unit
}

fn register_schema(
    spec: &OpenAPI,
    models_registry: &mut TypeRegistry,
    schema: &Schema,
    class_name: Option<String>,
    media_type: &str,
) -> Option<DelphiType> {
    let Some(r#type) = schema.r#type.as_ref() else {
        return None;
    };

    return match r#type.as_str() {
        "string" => {
            if schema.enum_.is_empty() {
                if let Some(format) = &schema.format {
                    match format.as_str() {
                        "byte" => Some(DelphiType::Binary(BinaryFormat::Base64)),
                        "binary" => Some(DelphiType::Binary(BinaryFormat::Binary)),
                        _ => Some(DelphiType::String),
                    }
                } else {
                    Some(DelphiType::String)
                }
            } else {
                let (gen_id, name) = class_name.or(schema.title.clone()).map_or_else(
                    || TypeRegistry::generate_name("Enum"),
                    |n| (UNRESOLVED_TYPE_ID, n),
                );

                let name = format!("T{}", capitalize(&name));
                let id = models_registry.find_enum_type_id_by_name(&name, "schema");
                if let Some(id) = id {
                    return Some(DelphiType::Enum(IrTypeIdOrName::Id(id)));
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

                Some(DelphiType::Enum(IrTypeIdOrName::Id(id)))
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

                let (schema, reference) = match items {
                    SchemaOrRef::Item(schema) => (Some(schema), None),
                    SchemaOrRef::Ref { reference } => (
                        spec.resolve_schema(reference),
                        reference.split("/").last().map(|v| v.to_owned()),
                    ),
                };

                let Some(schema) = schema else {
                    return None;
                };

                let inner_type =
                    register_schema(spec, models_registry, schema, reference, media_type)?;

                Some(DelphiType::List(Box::new(inner_type)))
            } else {
                Some(DelphiType::List(Box::new(DelphiType::Pointer)))
            }
        }
        "object" => {
            let (gen_id, name) = class_name.or(schema.title.clone()).map_or_else(
                || TypeRegistry::generate_name("Class"),
                |n| (UNRESOLVED_TYPE_ID, n),
            );

            let name = format!("T{}", capitalize(&name));
            let id = models_registry.find_class_type_id_by_name(&name, "schema");
            if let Some(id) = id {
                let class = models_registry.get_class_mut(&id).expect("");
                class.generate_json_support =
                    class.generate_json_support || media_type == "application/json";
                class.generate_xml_support =
                    class.generate_xml_support || media_type == "application/xml";

                return Some(DelphiType::Class(IrTypeIdOrName::Id(id)));
            }

            let mut class = DelphiClass::new(name, "schema");
            class.generate_json_support = media_type == "application/json";
            class.generate_xml_support = media_type == "application/xml";

            // Process properties
            for (prop_name, prop) in schema.properties.iter() {
                let prop_type = match &prop {
                    SchemaOrRef::Item(s) => Some(s),
                    SchemaOrRef::Ref { reference } => spec.resolve_schema(reference),
                };

                let Some(prop) = prop_type else {
                    continue;
                };

                let delphi_type = register_schema(
                    spec,
                    models_registry,
                    prop,
                    Some(prop_name.clone()),
                    media_type,
                );

                if let Some(delphi_type) = delphi_type {
                    let default_value = match delphi_type {
                        DelphiType::Binary(_)
                        | DelphiType::Boolean
                        | DelphiType::Enum(_)
                        | DelphiType::DateTime
                        | DelphiType::Double
                        | DelphiType::Float
                        | DelphiType::Integer
                        | DelphiType::String => prop.default.as_ref().map(|dv| dv.to_string()),
                        _ => None,
                    };

                    let xml_info = prop.xml.as_ref().map(|x| XmlInfo {
                        name: x.name.clone().unwrap_or_default(),
                        namespace: x.namespace.clone(),
                        prefix: x.prefix.clone(),
                        attribute: x.attribute,
                        wrapped: x.wrapped,
                    });

                    class.fields.push(DelphiField {
                        name: prop_name.clone(),
                        is_reference_type: matches!(
                            delphi_type,
                            DelphiType::List(_) | DelphiType::Class(_)
                        ),
                        field_type: delphi_type,
                        default_value,
                        visibility: DelphiVisibility::Public,
                        json_key: Some(prop_name.clone()),
                        is_required: schema.required.contains(prop_name),
                        xml_info: xml_info,
                        comment: None,
                    });
                }
            }

            let id = if gen_id == UNRESOLVED_TYPE_ID {
                models_registry.register_class(class)
            } else {
                models_registry.register_class_with_id(gen_id, class);

                gen_id
            };

            Some(DelphiType::Class(IrTypeIdOrName::Id(id)))
        }
        _ => None,
    };
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

fn capitalize(s: &str) -> String {
    let mut c = s.chars();

    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}
