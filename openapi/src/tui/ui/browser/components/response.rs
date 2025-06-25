use ratatui::prelude::*;

use crate::parser::types::*;

pub fn style_from_status_code(code: usize) -> Style {
    match code {
        100..=199 => Style::default().magenta(),
        200..=299 | 300..=399 => Style::default().green(),
        400..=499 => Style::default().yellow(),
        500..=599 => Style::default().red(),
        _ => Style::default(),
    }
}

pub fn ui<'a>(
    spec: &'a OpenAPI,
    response: &'a Response,
    name: String,
    indentation: usize,
    show_name: bool,
) -> Vec<Line<'a>> {
    let mut text = vec![Span::from(" ".repeat(indentation))];

    if show_name {
        if let Ok(value) = name.parse::<usize>() {
            text.push(Span::styled(
                format!("{name} "),
                style_from_status_code(value),
            ));
        } else {
            text.push(Span::from(format!("{name} ")));
        }
    }
    let header = Line::from(text);
    let description = Line::styled(&response.description, Style::default().dark_gray());

    let mut lines = if header.width() > 0 {
        vec![header, description]
    } else {
        vec![description]
    };

    let content_lines = response
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

                super::schema::ui(spec, schema, name, indentation + 2, true)
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
