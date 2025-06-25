use ratatui::prelude::*;

use crate::parser::types::*;

pub fn ui<'a>(
    spec: &'a OpenAPI,
    header: &'a Header,
    name: String,
    indentation: usize,
) -> Vec<Line<'a>> {
    let mut text = vec![Span::from(" ".repeat(indentation))];

    if indentation >= 2 {
        text.push(Span::from(format!("{name} ")));
    }

    if header.deprecated {
        text.push(Span::styled(" (deprecated)", Style::default().yellow()));
    }

    if header.required {
        text.push(Span::styled(" (required)", Style::default().red()));
    }

    let mut lines = vec![
        Line::from(text),
        Line::from(format!(
            "{}allow_empty_value: {}",
            " ".repeat(indentation),
            header.allow_empty_value
        )),
    ];

    if let Some(description) = &header.description {
        lines.push(Line::from(format!(
            "{}description: {}",
            " ".repeat(indentation),
            description
        )));
    }

    if let Some(style) = &header.style {
        lines.push(Line::from(format!(
            "{}style: {}",
            " ".repeat(indentation),
            style
        )));
    }

    if let Some(schema) = &header.schema {
        lines.push(Line::from(format!("{}schema:", " ".repeat(indentation))));

        let (name, schema, reference) = match schema {
            SchemaOrRef::Item(schema) => (None, Some(schema), None),
            SchemaOrRef::Ref { reference } => (
                reference.split("/").last(),
                spec.resolve_schema(&reference),
                Some(reference),
            ),
        };

        if let Some(schema) = schema {
            let name = name.map_or_else(|| "Custom Schema".to_owned(), |t| t.to_owned());

            lines.extend(super::schema::ui(spec, schema, name, indentation + 2, true));
        } else if let Some(reference) = reference {
            let name = name.map_or_else(|| "Custom Schema".to_owned(), |t| t.to_owned());

            lines.push(Line::from(Span::from(format!(
                "{}{name}: {reference}",
                " ".repeat(indentation)
            ))));
        } else {
            let name = name.map_or_else(|| "Custom Schema".to_owned(), |t| t.to_owned());

            lines.push(Line::from(Span::from(format!(
                "{}{name}: <unknown>",
                " ".repeat(indentation)
            ))));
        }
    }

    lines
}
