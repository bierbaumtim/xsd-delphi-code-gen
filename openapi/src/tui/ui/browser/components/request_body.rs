use ratatui::prelude::*;

use crate::parser::types::*;

pub fn ui<'a>(
    spec: &'a OpenAPI,
    request_body: &'a RequestBody,
    name: String,
    indentation: usize,
) -> Vec<Line<'a>> {
    let mut text = vec![Span::from(" ".repeat(indentation))];

    if indentation >= 2 {
        text.push(Span::from(format!("{name} ")));
    }

    if request_body.required {
        text.push(Span::styled(" (required)", Style::default().red()));
    }

    let mut lines = vec![Line::from(text)];

    if let Some(description) = &request_body.description {
        lines.push(Line::styled(description, Style::default().dark_gray()));
    }

    let content_lines = request_body
        .content
        .iter()
        .flat_map(|(media_type, media)| {
            let (name, schema, reference) = match media.schema.as_ref() {
                Some(SchemaOrRef::Item(schema)) => (None, Some(schema), None),
                Some(SchemaOrRef::Ref { reference }) => (
                    reference.split("/").last(),
                    spec.resolve_schema(&reference),
                    Some(reference),
                ),
                None => (None, None, None),
            };

            if let Some(schema) = schema {
                let name = name.map_or_else(|| "Custom Schema".to_owned(), |t| t.to_owned());

                super::schema::ui(spec, schema, name, reference, indentation + 2, true)
            } else if let Some(reference) = reference {
                vec![Line::from(Span::from(format!(
                    "{}{media_type}: {reference}",
                    " ".repeat(indentation),
                )))]
            } else {
                vec![Line::from(Span::from(format!(
                    "{}{media_type}: <unknown>",
                    " ".repeat(indentation),
                )))]
            }
        })
        .collect::<Vec<_>>();

    if !content_lines.is_empty() {
        lines.push(Line::from(format!("{}content:", " ".repeat(indentation))));
    }

    lines.extend(content_lines);

    lines
}
