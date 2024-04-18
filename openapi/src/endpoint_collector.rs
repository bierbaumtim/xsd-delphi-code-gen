use sw4rm_rs::{
    shared::{Operation, ParameterLocation, ParameterSchemaType, StringOrHttpCode},
    Spec,
};
use tera::Value;

use crate::{
    helper::{self, capitalize},
    models::{ClassType, Endpoint, EndpointArg, EnumType, Response as ResponseModel, Type},
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
            let status_codes = get_endpoint_responses(&o, spec, &name, class_types, enum_types);
            let request_body = get_endpoint_request_body(&o, spec, &name, class_types, enum_types)
                .unwrap_or_default();

            let endpoint = Endpoint {
                name,
                response_type,
                status_codes,
                args: get_endpoint_args(&o, spec),
                method: "GET".to_string(),
                path: k.to_string(),
                request_body,
            };

            endpoints.push(endpoint);
        }

        if let Some(o) = v.post {
            let name = get_endpoint_name(&o, k, "Post");
            let response_type =
                get_endpoint_response_type(&o, spec, &name, class_types, enum_types);
            let status_codes = get_endpoint_responses(&o, spec, &name, class_types, enum_types);
            let request_body = get_endpoint_request_body(&o, spec, &name, class_types, enum_types)
                .unwrap_or_default();

            let endpoint = Endpoint {
                name,
                response_type,
                status_codes,
                args: get_endpoint_args(&o, spec),
                method: "POST".to_string(),
                path: k.to_string(),
                request_body,
            };

            endpoints.push(endpoint);
        }

        if let Some(o) = v.put {
            let name = get_endpoint_name(&o, k, "Put");
            let response_type =
                get_endpoint_response_type(&o, spec, &name, class_types, enum_types);
            let status_codes = get_endpoint_responses(&o, spec, &name, class_types, enum_types);
            let request_body = get_endpoint_request_body(&o, spec, &name, class_types, enum_types)
                .unwrap_or_default();

            let endpoint = Endpoint {
                name,
                response_type,
                status_codes,
                args: get_endpoint_args(&o, spec),
                method: "PUT".to_string(),
                path: k.to_string(),
                request_body,
            };

            endpoints.push(endpoint);
        }

        if let Some(o) = v.delete {
            let name = get_endpoint_name(&o, k, "Delete");
            let response_type =
                get_endpoint_response_type(&o, spec, &name, class_types, enum_types);
            let status_codes = get_endpoint_responses(&o, spec, &name, class_types, enum_types);
            let request_body = get_endpoint_request_body(&o, spec, &name, class_types, enum_types)
                .unwrap_or_default();

            let endpoint = Endpoint {
                name,
                response_type,
                status_codes,
                args: get_endpoint_args(&o, spec),
                method: "DELETE".to_string(),
                path: k.to_string(),
                request_body,
            };

            endpoints.push(endpoint);
        }
    }

    endpoints
}

fn get_endpoint_name(operation: &Operation, path: &str, method: &str) -> String {
    match operation.operation_id.as_ref() {
        Some(name) => {
            if name.contains('/') {
                format!(
                    "{}{}",
                    method,
                    capitalize(
                        path.trim_end_matches('/')
                            .split('/')
                            .last()
                            .unwrap()
                            .to_string()
                            .as_str()
                    )
                )
            } else {
                sanitize_operation_id(name)
            }
        }
        None => format!(
            "{}{}",
            method,
            capitalize(
                path.trim_end_matches('/')
                    .split('/')
                    .last()
                    .unwrap()
                    .to_string()
                    .as_str()
            )
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
        .find(|r| match r.0 {
            StringOrHttpCode::String(s) => s.starts_with('2'),
            StringOrHttpCode::StatusCode(c) => *c >= 200 && *c < 300,
        })
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

fn get_endpoint_responses(
    operation: &Operation,
    spec: &Spec,
    endpoint_name: &str,
    class_types: &mut Vec<ClassType>,
    enum_types: &mut Vec<EnumType>,
) -> Vec<ResponseModel> {
    let mut responses = vec![];

    for (k, v) in &operation.responses {
        let v = match v.resolve(spec) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let response = ResponseModel {
            status_code: match k {
                StringOrHttpCode::String(s) => s.to_string(),
                StringOrHttpCode::StatusCode(c) => c.to_string(),
            },
            type_: v
                .content
                .get("application/json")
                .cloned()
                .and_then(|m| m.schema)
                .and_then(|s| s.resolve(spec).ok())
                .and_then(|s| {
                    schema_collector::schema_to_type(
                        &s,
                        endpoint_name,
                        spec,
                        None,
                        class_types,
                        enum_types,
                    )
                })
                .map_or(Type::default(), |(n, c, e)| Type {
                    name: n,
                    is_class: c,
                    is_enum: e,
                }),
            is_list_type: false,
        };

        responses.push(response);
    }

    responses.sort_by_key(|r| r.status_code.clone());
    responses
}

fn get_endpoint_args(operation: &Operation, spec: &Spec) -> Vec<EndpointArg> {
    let mut args = operation
        .parameters
        .iter()
        .filter_map(|p| {
            p.resolve(spec).ok().map(|p| {
                let name = capitalize(&p.name.clone().unwrap_or_default());

                let s_type_name = match p.schema_type {
                    Some(ParameterSchemaType::Boolean) => "boolean".to_string(),
                    Some(ParameterSchemaType::Integer) => "integer".to_string(),
                    Some(ParameterSchemaType::Number) => "double".to_string(),
                    Some(ParameterSchemaType::String) => "string".to_string(),
                    _ => "".to_string(),
                };

                let type_name = match &p.schema {
                    Some(s) => s.resolve(spec).ok().and_then(|s| {
                        s.schema_type
                            .as_ref()
                            .map(|t| helper::schema_type_to_base_type(t, &None))
                    }),
                    None => None,
                };

                let arg_type = match p.location.unwrap_or_default() {
                    ParameterLocation::Query => "query".to_owned(),
                    ParameterLocation::Path => "path".to_owned(),
                    ParameterLocation::Body => "body".to_owned(),
                    ParameterLocation::Header => todo!(),
                    ParameterLocation::FormData => todo!(),
                    ParameterLocation::Cookie => todo!(),
                };

                EndpointArg {
                    name,
                    type_name: type_name.unwrap_or(s_type_name),
                    arg_type,
                    is_required: p.required.unwrap_or_default(),
                    default_value: match &p.default {
                        Some(Value::String(s)) => s.to_string(),
                        Some(Value::Bool(s)) => {
                            if *s {
                                "true".to_string()
                            } else {
                                "false".to_string()
                            }
                        }
                        Some(Value::Number(n)) => n.to_string(),
                        Some(d) => d.to_string(),
                        None => "".to_string(),
                    },
                }
            })
        })
        .collect::<Vec<EndpointArg>>();

    args.sort_by_key(|a| a.default_value.is_empty());

    args
}

fn get_endpoint_request_body(
    operation: &Operation,
    spec: &Spec,
    endpoint_name: &str,
    class_types: &mut Vec<ClassType>,
    enum_types: &mut Vec<EnumType>,
) -> Option<Type> {
    let name = endpoint_name.to_string() + "RequestBody";

    operation
        .request_body
        .as_ref()
        .and_then(|r| r.resolve(spec).ok())
        .and_then(|r| r.content.get("application/json").cloned())
        .and_then(|m| m.schema)
        .and_then(|s| s.resolve(spec).ok())
        .and_then(|s| {
            schema_collector::schema_to_type(&s, &name, spec, None, class_types, enum_types)
        })
        .map(|(n, c, e)| Type {
            name: n,
            is_class: c,
            is_enum: e,
        })
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
