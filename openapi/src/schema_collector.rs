use sw4rm_rs::{
    shared::{Schema, SchemaType},
    RefOr, Reference, Spec,
};
use tera::Value;

use crate::models::{ClassType, EnumType, EnumVariant, Property};
use crate::{
    helper::{capitalize, get_enum_variant_prefix, sanitize_name, schema_type_to_base_type},
    models::Type,
};

pub(crate) fn collect_types(
    spec: &Spec,
    prefix: Option<String>,
) -> (Vec<ClassType>, Vec<EnumType>) {
    let mut class_types = vec![];
    let mut enum_types = vec![];

    for (k, v) in spec.schemas() {
        let s = match v.resolve(spec) {
            Ok(s) => s,
            Err(_) => continue,
        };

        schema_to_type(
            &s,
            k.as_str(),
            spec,
            prefix.clone(),
            &mut class_types,
            &mut enum_types,
        );
    }

    (class_types, enum_types)
}

pub(crate) fn schema_to_type(
    schema: &Schema,
    name: &str,
    spec: &Spec,
    prefix: Option<String>,
    class_types: &mut Vec<ClassType>,
    enum_types: &mut Vec<EnumType>,
) -> Option<(String, bool, bool)> {
    match schema.schema_type {
        Some(SchemaType::String) if !schema.enum_values.is_empty() => {
            let enum_type = build_enum_type(name, &schema.enum_values, prefix.clone());
            let name = enum_type.name.clone();

            if !enum_types.iter().any(|e| *e == enum_type) {
                enum_types.push(enum_type);
            }

            Some((name, false, true))
        }
        Some(SchemaType::Object) => {
            let properties = schema
                .properties
                .iter()
                .filter_map(|(k, v)| {
                    v.resolve(spec).ok().and_then(|s| {
                        let (type_name, is_reference_type, is_enum_type) =
                            s.schema_type.as_ref().map(|t| match t {
                                SchemaType::String if !s.enum_values.is_empty() => {
                                    let enum_type =
                                        build_enum_type(k, &s.enum_values, prefix.clone());
                                    let name = enum_type.name.clone();

                                    if !enum_types.iter().any(|e| *e == enum_type) {
                                        enum_types.push(enum_type);
                                    }

                                    (name, false, true)
                                }
                                SchemaType::Array => {
                                    let items = s
                                        .items
                                        .as_ref()
                                        .expect("Array must have items property set");
                                    let item_schema = items
                                        .resolve(spec)
                                        .expect("Type of array items must be resolved");

                                    let (name, is_class, is_enum) = schema_to_type(
                                        &item_schema,
                                        &match items {
                                            RefOr::Reference { reference_path } => {
                                                let reference =
                                                    Reference::try_from(reference_path.clone())
                                                        .unwrap();

                                                reference.name
                                            }
                                            _ => k.to_owned() + "Item",
                                        },
                                        spec,
                                        prefix.clone(),
                                        class_types,
                                        enum_types,
                                    )
                                    .expect("Type of array items must be resolved");

                                    (name, is_class, is_enum)
                                }
                                SchemaType::Object => {
                                    (s.title.clone().unwrap_or(k.to_string()), true, false)
                                }
                                _ => (schema_type_to_base_type(t, &s.format), false, false),
                            })?;

                        Some(Property {
                            name: capitalize(k),
                            key: k.to_owned(),
                            is_list_type: s.schema_type.is_some_and(|t| t == SchemaType::Array),
                            type_: Type {
                                name: type_name,
                                is_class: is_reference_type,
                                is_enum: is_enum_type,
                            },
                        })
                    })
                })
                .collect::<Vec<Property>>();

            let class_type = ClassType {
                name: capitalize(name),
                needs_destructor: properties.iter().any(|p| p.type_.is_class),
                properties,
            };

            if !class_types.iter().any(|c| *c == class_type) {
                class_types.push(class_type);
            }

            Some((capitalize(name), true, false))
        }
        Some(SchemaType::Array) => None,
        Some(t) => Some((schema_type_to_base_type(&t, &schema.format), false, false)),
        _ => None,
    }
}

fn build_enum_type(name: &str, variants: &[Value], prefix: Option<String>) -> EnumType {
    let name = capitalize(name);
    let variant_prefix = get_enum_variant_prefix(&name, &prefix.unwrap_or_default());

    EnumType {
        name: name.clone(),
        variants: variants
            .iter()
            .filter_map(|v| {
                v.as_str().map(|s| EnumVariant {
                    name: variant_prefix.clone() + &sanitize_name(&capitalize(s)),
                    key: s.to_owned(),
                })
            })
            .collect::<Vec<EnumVariant>>(),
    }
}
