use std::u16;

use ratatui::{prelude::*, widgets::*};

use crate::tui::state::{App, ComponentsRegion};

pub mod header;
pub mod parameter;
pub mod request_body;
pub mod response;
pub mod schema;

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
    .block(
        if app.components_focused_region == ComponentsRegion::Navigation {
            Block::bordered().border_style(Style::default().blue())
        } else {
            Block::bordered()
        }
        .title("Navigation"),
    )
    .style(Style::default().white())
    .highlight_style(Style::default().blue().bold())
    .highlight_symbol(">>")
    .repeat_highlight_symbol(false)
    .direction(ListDirection::TopToBottom);

    f.render_stateful_widget(list, area, &mut app.components_navigation_list_state);

    if app.components_navigation_list_state.selected().is_none() {
        app.components_navigation_list_state.select_first();
    }
}

fn render_list(f: &mut Frame, app: &mut App, area: Rect) {
    let selected_index = app.components_navigation_list_state.selected().unwrap_or(0);

    let items = match selected_index {
        0 => {
            let items = app.get_parameter_list_items();

            items
                .iter()
                .map(|(name, _)| ListItem::new(format!("{name}")))
                .collect::<Vec<_>>()
        }
        // 1 => render_request_bodies(f, app, area),
        2 => {
            let items = app.get_response_list_items();

            items
                .iter()
                .map(|(name, response)| {
                    ListItem::new(format!("{}: {}", name, response.description))
                })
                .collect::<Vec<_>>()
        }
        3 => {
            let items = app.get_schemas_list_items();

            items
                .iter()
                .map(|(name, schema)| {
                    let content = format!(
                        "{}: {}",
                        name,
                        schema.title.as_ref().cloned().unwrap_or_default()
                    );

                    ListItem::new(content)
                })
                .collect()
        }
        // 4 => render_headers(f, app, area),
        _ => vec![],
    };
    let title = match selected_index {
        0 => "Parameters",
        1 => "Request Bodies",
        2 => "Responses",
        3 => "Schemas",
        4 => "Headers",
        _ => "Unknown",
    };

    let select_item_index = app
        .components_list_state
        .selected()
        .map_or(0, |i| i.checked_add(1).unwrap_or(1));
    let items_len = items.len();

    let list = List::new(items)
        .block(
            if app.components_focused_region == ComponentsRegion::List {
                Block::bordered().border_style(Style::default().blue())
            } else {
                Block::bordered()
            }
            .title(title)
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
    let Some(spec) = app.spec.as_ref() else {
        return;
    };
    let Some(index) = app.components_list_state.selected() else {
        return;
    };
    let selected_region = app.components_navigation_list_state.selected().unwrap_or(0);

    let lines: Vec<Line>;
    let title: String;
    match selected_region {
        0 => {
            let Some((name, param)) = app.get_parameter_at(index) else {
                return;
            };
            let name = name.clone();

            title = name.clone();
            lines = parameter::ui(spec, &param, name, 0);
        }
        // 1 => lines = request_body::ui(app),
        2 => {
            let Some((name, response)) = app.get_response_at(index) else {
                return;
            };
            let name = name.clone();

            title = format!("{} - {}", name, response.description);
            lines = response::ui(spec, response, name, 0);
        }
        3 => {
            let Some((name, component)) = app.get_schema_at(index) else {
                return;
            };
            let name = name.clone();

            let Some(type_) = component.r#type.as_ref() else {
                return;
            };

            title = format!("{} - {}", name, type_);
            lines = schema::ui(spec, &component, name, 0, true);
        }
        // 4 => lines = header::ui(app),
        _ => {
            title = String::new();
            lines = vec![Line::from("No details available")];
        }
    }

    let text = Text::from(lines);
    let text = Paragraph::new(text)
        .block(
            if app.components_focused_region == ComponentsRegion::Details {
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
