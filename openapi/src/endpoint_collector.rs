use sw4rm_rs::{
    shared::{Operation, ParameterLocation, ParameterSchemaType, StringOrHttpCode},
    Spec,
};

use crate::{
    helper::{self, capitalize},
    models::{ClassType, Endpoint, EndpointArg, EnumType, Type},
    schema_collector,
};

pub(crate) fn collect_endpoints(
    spec: &Spec,
    class_types: &mut Vec<ClassType>,
    enum_types: &mut Vec<EnumType>,
) -> Vec<Endpoint> {
    let mut endpoints = vec![];

    for (k, v) in &spec.paths {
        let v = match v.resolve(spec) {
            Ok(v) => v,
            Err(_) => continue,
        };

        if let Some(o) = v.get {
            let name = get_endpoint_name(&o, k, "Get");
            let response_type =
                get_endpoint_response_type(&o, spec, &name, class_types, enum_types);

            let endpoint = Endpoint {
                name,
                response_type,
                args: get_endpoint_args(&o, spec),
            };

            endpoints.push(endpoint);
        }

        if let Some(o) = v.post {
            let name = get_endpoint_name(&o, k, "Post");
            let response_type =
                get_endpoint_response_type(&o, spec, &name, class_types, enum_types);

            let endpoint = Endpoint {
                name,
                response_type,
                args: get_endpoint_args(&o, spec),
            };

            endpoints.push(endpoint);
        }

        if let Some(o) = v.put {
            let name = get_endpoint_name(&o, k, "Put");
            let response_type =
                get_endpoint_response_type(&o, spec, &name, class_types, enum_types);

            let endpoint = Endpoint {
                name,
                response_type,
                args: get_endpoint_args(&o, spec),
            };

            endpoints.push(endpoint);
        }

        if let Some(o) = v.delete {
            let name = get_endpoint_name(&o, k, "Delete");
            let response_type =
                get_endpoint_response_type(&o, spec, &name, class_types, enum_types);

            let endpoint = Endpoint {
                name,
                response_type,
                args: get_endpoint_args(&o, spec),
            };

            endpoints.push(endpoint);
        }
    }

    endpoints
}

fn get_endpoint_name(operation: &Operation, path: &str, method: &str) -> String {
    match operation.operation_id.as_ref() {
        Some(name) => {
            if name.contains("/") {
                format!(
                    "{}{}",
                    method,
                    path.trim_end_matches("/")
                        .split("/")
                        .last()
                        .unwrap()
                        .to_string()
                )
            } else {
                sanitize_operation_id(name)
            }
        }
        None => format!(
            "{}{}",
            method,
            path.trim_end_matches("/")
                .split("/")
                .last()
                .unwrap()
                .to_string()
        ),
    }
}

fn get_endpoint_response_type(
    operation: &Operation,
    spec: &Spec,
    endpoint_name: &str,
    class_types: &mut Vec<ClassType>,
    enum_types: &mut Vec<EnumType>,
) -> Type {
    let (response_type, is_class, is_enum) = operation
        .responses
        .iter()
        .filter(|r| match r.0 {
            StringOrHttpCode::String(s) => s.starts_with("2"),
            StringOrHttpCode::StatusCode(c) => *c >= 200 && *c < 300,
        })
        .next()
        .and_then(|r| r.1.resolve(spec).ok())
        .and_then(|r| r.content.get("application/json").cloned())
        .and_then(|m| m.schema)
        .and_then(|s| s.resolve(spec).ok())
        .and_then(|s| {
            schema_collector::schema_to_type(&s, endpoint_name, spec, None, class_types, enum_types)
        })
        .unwrap_or(("none".to_string(), false, false));

    Type {
        name: response_type,
        is_class,
        is_enum,
    }
}

fn get_endpoint_args(operation: &Operation, spec: &Spec) -> Vec<EndpointArg> {
    operation
        .parameters
        .iter()
        .filter_map(|p| {
            p.resolve(spec).ok().and_then(|p| {
                let name = capitalize(&p.name.clone().unwrap_or_default());

                let s_type_name = match p.schema_type {
                    Some(ParameterSchemaType::Boolean) => "boolean".to_string(),
                    Some(ParameterSchemaType::Integer) => "integer".to_string(),
                    Some(ParameterSchemaType::Number) => "double".to_string(),
                    Some(ParameterSchemaType::String) => "string".to_string(),
                    _ => "".to_string(),
                };

                let type_name = match &p.schema {
                    Some(s) => s.resolve(spec).ok().and_then(|s| match &s.schema_type {
                        Some(t) => Some(helper::schema_type_to_base_type(&t, &None)),
                        None => None,
                    }),
                    None => None,
                };

                let arg_type = match p.location.clone().unwrap_or_default() {
                    ParameterLocation::Query => "query".to_owned(),
                    ParameterLocation::Path => "path".to_owned(),
                    ParameterLocation::Header => todo!(),
                    ParameterLocation::FormData => todo!(),
                    ParameterLocation::Body => todo!(),
                    ParameterLocation::Cookie => todo!(),
                };

                Some(EndpointArg {
                    name,
                    type_name: type_name.unwrap_or(s_type_name),
                    arg_type,
                })
            })
        })
        .collect::<Vec<EndpointArg>>()
}

fn sanitize_operation_id(name: &str) -> String {
    let chars = name.chars();

    let mut next_char_upper = false;
    let mut sanitized = String::with_capacity(name.len());

    for (i, c) in chars.enumerate() {
        if c.is_alphanumeric() {
            if i == 0 || next_char_upper {
                sanitized.push(c.to_ascii_uppercase());
                next_char_upper = false;
            } else {
                sanitized.push(c);
            }
        } else {
            next_char_upper = true;
            continue;
        }
    }

    sanitized
}
