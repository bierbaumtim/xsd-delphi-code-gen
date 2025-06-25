use ratatui::prelude::*;

use crate::parser::types::*;

pub fn ui<'a>(
    spec: &'a OpenAPI,
    param: &'a Parameter,
    name: String,
    indentation: usize,
) -> Vec<Line<'a>> {
    let mut text = vec![Span::from(" ".repeat(indentation))];

    if indentation >= 2 {
        text.push(Span::from(format!("{name} ")));
    }

    text.push(Span::from(format!("in: {}", param.in_.as_str())));

    if param.deprecated {
        text.push(Span::styled(" (deprecated)", Style::default().yellow()));
    }

    if param.required {
        text.push(Span::styled(" (required)", Style::default().red()));
    }

    let mut lines = vec![
        Line::from(text),
        Line::from(format!(
            "{}allow_empty_value: {}",
            " ".repeat(indentation),
            param.allow_empty_value
        )),
    ];

    if let Some(description) = &param.description {
        lines.push(Line::from(format!(
            "{}description: {}",
            " ".repeat(indentation),
            description
        )));
    }

    if let Some(style) = &param.style {
        lines.push(Line::from(format!(
            "{}style: {}",
            " ".repeat(indentation),
            style
        )));
    }

    if let Some(schema) = &param.schema {
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
