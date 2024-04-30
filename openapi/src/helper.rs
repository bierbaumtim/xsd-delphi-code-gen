use sw4rm_rs::shared::SchemaType;

pub(crate) fn capitalize(value: &str) -> String {
    let mut c = value.chars();

    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

pub(crate) fn get_enum_variant_prefix(name: &str, type_prefix: &str) -> String {
    let prefixed_type_name = type_prefix.to_owned() + name;

    prefixed_type_name
        .chars()
        .filter(|c| c.is_uppercase())
        .collect::<String>()
        .to_lowercase()
}

pub(crate) fn sanitize_name(name: &str) -> String {
    name.replace(['-', '.'], "_")
}

pub(crate) fn schema_type_to_base_type(
    schema_type: SchemaType,
    format: &Option<String>,
) -> String {
    match schema_type {
        SchemaType::String => match format {
            Some(f) => match f.as_str() {
                "date" | "date-time" => "datetime".to_string(),
                _ => "string".to_string(),
            },
            None => "string".to_string(),
        },
        SchemaType::Integer => "integer".to_string(),
        SchemaType::Number => "double".to_string(),
        SchemaType::Boolean => "boolean".to_string(),
        _ => String::new(),
    }
}
