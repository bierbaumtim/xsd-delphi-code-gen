use ratatui::prelude::*;

use crate::parser::types::*;

pub fn ui<'a>(
    spec: &'a OpenAPI,
    schema: &'a Schema,
    name: String,
    indentation: usize,
    increase_indentation: bool,
) -> Vec<Line<'a>> {
    let Some(type_) = schema.r#type.as_ref() else {
        return vec![Line::from(Span::from(format!(
            "{}{}: <unknown type>",
            " ".repeat(indentation),
            name
        )))];
    };

    match type_.as_str() {
        "object" => {
            let mut lines = vec![];

            if indentation >= 2 {
                lines.push(Line::from(Span::from(format!(
                    "{}{name}: object",
                    " ".repeat(indentation)
                ))));
            }

            let content_lines = schema
                .properties
                .iter()
                .flat_map(|(key, value)| {
                    let (schema, reference) = match value {
                        SchemaOrRef::Item(schema) => (Some(schema), None),
                        SchemaOrRef::Ref { reference } => {
                            (spec.resolve_schema(reference), Some(reference))
                        }
                    };

                    if let Some(schema) = schema {
                        let indentation = if increase_indentation {
                            indentation + 2
                        } else {
                            indentation
                        };
                        ui(spec, schema, key.clone(), indentation, true)
                    } else if let Some(reference) = reference {
                        vec![Line::from(Span::from(format!(
                            "{}{key}: {reference}",
                            " ".repeat(indentation)
                        )))]
                    } else {
                        vec![Line::from(Span::from(format!(
                            "{}{key}: <unknown>",
                            " ".repeat(indentation)
                        )))]
                    }
                })
                .collect::<Vec<_>>();

            lines.extend(content_lines);

            lines
        }
        "array" => {
            let Some(items) = &schema.items else {
                return vec![];
            };

            let mut lines = vec![];

            if indentation >= 2 {
                lines.push(Line::from(Span::from(format!(
                    "{}{name}: array",
                    " ".repeat(indentation)
                ))));
            }

            let item_schema: &SchemaOrRef = items;
            let (schema, reference) = match item_schema {
                SchemaOrRef::Item(schema) => (Some(schema), None),
                SchemaOrRef::Ref { reference } => (spec.resolve_schema(reference), Some(reference)),
            };

            if let Some(schema) = schema {
                let indentation = if increase_indentation {
                    indentation + 2
                } else {
                    indentation
                };
                lines.extend(ui(spec, schema, name, indentation, true));
            } else if let Some(reference) = reference {
                lines.push(Line::from(Span::from(format!(
                    "{}{name}: {reference}",
                    " ".repeat(indentation)
                ))));
            } else {
                lines.push(Line::from(Span::from(format!(
                    "{}{name}: <unknown>",
                    " ".repeat(indentation)
                ))));
            }

            lines
        }
        "integer" | "number" => {
            let format = schema
                .format
                .as_ref()
                .map(|f| Span::styled(format!(" (format: {})", f), Style::default().dark_gray()));

            let default = schema
                .default
                .as_ref()
                .map(|d| Span::styled(format!(" (default: {})", d), Style::default().dark_gray()));

            let mut result = vec![];

            // if !length_range.content.is_empty() {
            //     result.push(length_range);
            // }
            if let Some(format) = format {
                result.push(format);
            }
            if schema.nullable {
                result.push(Span::styled(" (nullable)", Style::default().dark_gray()));
            }
            if let Some(default) = default {
                result.push(default);
            }
            if !schema.enum_.is_empty() {
                result.push(Span::styled(
                    format!(
                        " (enum: {})",
                        schema
                            .enum_
                            .iter()
                            .map(|v| { format!("{}", v) })
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                    Style::default().dark_gray(),
                ));
            }

            let mut text = vec![
                Span::from(" ".repeat(indentation)),
                Span::from(name),
                Span::from(": "),
                Span::styled(type_, Style::default().dark_gray()),
            ];
            text.extend(result);

            vec![Line::from(text)]
        }
        "string" => {
            let length_range = match (schema.min_length, schema.max_length) {
                (Some(min), Some(max)) => format!(" ({}-{})", min, max),
                (Some(min), None) => format!(" (min {})", min),
                (None, Some(max)) => format!(" (max {})", max),
                (None, None) => String::new(),
            };
            let length_range = Span::styled(length_range, Style::default().dark_gray());

            let pattern = schema
                .pattern
                .as_ref()
                .map(|p| Span::styled(format!(" (pattern: {})", p), Style::default().dark_gray()));

            let format = schema
                .format
                .as_ref()
                .map(|f| Span::styled(format!(" (format: {})", f), Style::default().dark_gray()));

            let default = schema
                .default
                .as_ref()
                .map(|d| Span::styled(format!(" (default: {})", d), Style::default().dark_gray()));

            let mut result = vec![];
            if let Some(pattern) = pattern {
                result.push(pattern);
            }
            if !length_range.content.is_empty() {
                result.push(length_range);
            }
            if let Some(format) = format {
                result.push(format);
            }
            if schema.nullable {
                result.push(Span::styled(" (nullable)", Style::default().dark_gray()));
            }
            if let Some(default) = default {
                result.push(default);
            }
            if !schema.enum_.is_empty() {
                let trailing = if schema.enum_.len() > 10 {
                    String::from(" ...")
                } else {
                    String::new()
                };

                result.push(Span::styled(
                    format!(
                        " (enum: {}{})",
                        schema
                            .enum_
                            .iter()
                            .take(10)
                            .map(|v| { format!("{}", v) })
                            .collect::<Vec<_>>()
                            .join(", "),
                        trailing
                    ),
                    Style::default().dark_gray(),
                ));
            }

            let mut text = vec![
                Span::from(" ".repeat(indentation)),
                Span::from(name),
                Span::from(": "),
                Span::styled(type_, Style::default().dark_gray()),
            ];
            text.extend(result);

            vec![Line::from(text)]
        }
        _ => {
            let text = vec![
                Span::from(" ".repeat(indentation)),
                Span::from(name),
                Span::from(": "),
                Span::styled(type_, Style::default().dark_gray()),
            ];

            vec![Line::from(text)]
        }
    }
}
