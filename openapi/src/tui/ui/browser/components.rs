use std::u16;

use ratatui::{prelude::*, widgets::*};

use crate::{parser::types::*, tui::state::App};

pub fn ui(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::new(
        Direction::Horizontal,
        [
            Constraint::Length(20),
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ],
    )
    .split(area);

    render_navigation(f, app, chunks[0]);
    render_list(f, app, chunks[1]);
    render_details(f, app, chunks[2]);
}

fn render_navigation(f: &mut Frame, app: &mut App, area: Rect) {
    let list = List::new(vec![
        ListItem::new("Parameters"),
        ListItem::new("Request Bodies"),
        ListItem::new("Responses"),
        ListItem::new("Schemas"),
        ListItem::new("Headers"),
    ])
    .block(Block::bordered().title("Navigation"))
    .style(Style::default().white())
    .highlight_style(Style::default().blue().bold())
    .highlight_symbol(">>")
    .repeat_highlight_symbol(false)
    .direction(ListDirection::TopToBottom);

    f.render_widget(list, area);
}

fn render_list(f: &mut Frame, app: &mut App, area: Rect) {
    let items = app.get_components_list_items();

    let items: Vec<ListItem> = items
        .iter()
        .filter_map(|(name, schema)| {
            let content = format!(
                "{}: {}",
                name,
                schema.title.as_ref().cloned().unwrap_or_default()
            );
            let item = ListItem::new(content);

            Some(item)
        })
        .collect();

    let select_item_index = app
        .components_list_state
        .selected()
        .map_or(0, |i| i.checked_add(1).unwrap_or(1));
    let items_len = items.len();

    let list = List::new(items)
        .block(
            if !app.components_details_focused {
                Block::bordered().border_style(Style::default().blue())
            } else {
                Block::bordered()
            }
            .title("Components - Schemas")
            .title_bottom(
                Line::from(format!("{}/{}", select_item_index, items_len)).right_aligned(),
            ),
        )
        .style(Style::default().white())
        .highlight_style(Style::default().blue().bold())
        .highlight_symbol(">> ")
        .repeat_highlight_symbol(false)
        .direction(ListDirection::TopToBottom);

    f.render_stateful_widget(list, area, &mut app.components_list_state);

    if app.components_list_state.selected().is_none() {
        app.components_list_state.select_first();
    }
}

fn render_details(f: &mut Frame, app: &mut App, area: Rect) {
    let Some(index) = app.components_list_state.selected() else {
        return;
    };
    let Some((name, component)) = app.get_component_at(index) else {
        return;
    };
    let name = name.clone();
    let component = component.clone();

    let Some(type_) = component.r#type.as_ref() else {
        return;
    };

    let title = format!("{} - {}", name, type_);

    let lines = if let Some(spec) = &app.spec {
        render_schema(spec, &component, name, 1, false)
    } else {
        vec![]
    };
    let text = Text::from(lines);
    let text = Paragraph::new(text)
        .block(
            if app.components_details_focused {
                Block::bordered().border_style(Style::default().blue())
            } else {
                Block::bordered()
            }
            .title(title),
        )
        .wrap(Wrap { trim: false });

    let lines = text.line_count(area.width);
    let scroll_pos = app.components_details_scroll_pos.clamp(
        0,
        u16::try_from(lines)
            .unwrap_or(u16::MAX)
            .saturating_sub(area.height),
    );

    let text = text.scroll((scroll_pos, 0));

    f.render_widget(text, area);

    // Add scrollbar
    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"));
    app.components_details_scroll_pos = scroll_pos;
    let mut scroll_bar_state =
        ScrollbarState::new(lines).position(app.components_details_scroll_pos as usize);

    f.render_stateful_widget(
        scrollbar,
        area.inner(Margin {
            // using an inner vertical margin of 1 unit makes the scrollbar inside the block
            vertical: 1,
            horizontal: 0,
        }),
        &mut scroll_bar_state,
    );
}

pub fn render_schema<'a>(
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
                        render_schema(spec, schema, key.clone(), indentation, true)
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
                lines.extend(render_schema(spec, schema, name, indentation, true));
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
