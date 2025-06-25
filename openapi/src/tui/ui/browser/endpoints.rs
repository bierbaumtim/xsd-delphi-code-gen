use ratatui::{prelude::*, widgets::*};

use crate::{
    parser::types::*,
    tui::{
        state::{App, EndpointTab},
        ui::browser::components,
    },
};

pub fn ui(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::new(
        Direction::Horizontal,
        [Constraint::Percentage(50), Constraint::Percentage(50)],
    )
    .split(area);

    render_list(f, app, chunks[0]);
    render_details(f, app, chunks[1]);

    if app.endpoints_list_state.selected() != app.endpoints_selected_index {
        app.endpoints_selected_index = app.endpoints_list_state.selected();
        app.endpoints_details_body_scroll_pos = 0;
        app.endpoints_details_parameters_scroll_pos = 0;
        app.endpoints_details_responses_scroll_pos = 0;
    }
}

fn render_list(f: &mut Frame, app: &mut App, area: Rect) {
    let items = app.get_endpoints_list_items();

    let items = items.iter().map(|(color, path, method, _, op)| {
        let mixed_line = Line::from(vec![
            Span::styled(
                format!(
                    " ({}) - ",
                    op.tags
                        .first()
                        .cloned()
                        .unwrap_or_else(|| "No Tag".to_owned())
                ),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("[{method}]"),
                Style::default()
                    .fg(color.clone())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::from(format!(" {path} ")),
        ]);

        let text = Text::from(vec![
            mixed_line,
            Line::styled(
                format!("  -  {}", op.summary.clone().unwrap_or_default()),
                Style::default().fg(Color::DarkGray),
            ),
        ]);

        ListItem::new(text)
    });

    let select_item_index = app
        .endpoints_list_state
        .selected()
        .map_or(0, |i| i.checked_add(1).unwrap_or(1));
    let items_len = items.len();

    let list = List::new(items)
        .block(
            if app.endpoints_details_focused {
                Block::bordered()
            } else {
                Block::bordered().border_style(Style::default().bold().blue())
            }
            .title("Endpoints")
            .title_bottom(
                Line::from(format!("{}/{}", select_item_index, items_len)).right_aligned(),
            ),
        )
        .style(Style::default().white())
        .highlight_style(Style::default().blue().bold())
        .highlight_symbol(">>")
        .repeat_highlight_symbol(false)
        .direction(ListDirection::TopToBottom);

    f.render_stateful_widget(list, area, &mut app.endpoints_list_state);
}

fn render_details(f: &mut Frame, app: &mut App, area: Rect) {
    let Some(index) = app.endpoints_list_state.selected() else {
        return;
    };
    let Some((color, path, method, _, op)) = app.get_endpoint_at(index) else {
        return;
    };
    let op = op.clone();

    let main_layout = Layout::new(
        Direction::Vertical,
        [Constraint::Length(3), Constraint::Percentage(100)],
    )
    .split(area);

    let header = Paragraph::new(Line::from(vec![
        Span::styled(method, Style::default().bold().fg(color)),
        Span::styled(format!(" - {path}"), Style::default().fg(Color::DarkGray)),
    ]))
    .block(if let Some(summary) = &op.summary {
        Block::bordered().title(summary.as_str())
    } else {
        Block::bordered()
    });

    f.render_widget(header, main_layout[0]);

    if app.endpoints_details_layout == 0 {
        let layout_1_main = Layout::new(
            Direction::Vertical,
            [Constraint::Percentage(25), Constraint::Percentage(75)],
        )
        .split(main_layout[1]);

        let layout_1_v = Layout::new(
            Direction::Vertical,
            [Constraint::Length(3), Constraint::Percentage(100)],
        )
        .split(match &op.description {
            Some(description) if !description.is_empty() => layout_1_main[1],
            _ => main_layout[1],
        });

        if let Some(description) = &op.description {
            if !description.is_empty() {
                let description = Paragraph::new(Line::from(description.as_str()))
                    .block(Block::bordered().title("Description"))
                    .wrap(Wrap { trim: true });

                f.render_widget(description, layout_1_main[0]);
            }
        }

        let mut tabs = vec![];

        if let Some(body) = &op.request_body {
            let body = match body {
                RequestBodyOrRef::Item(body) => Some(body),
                RequestBodyOrRef::Ref { reference } => {
                    app.spec.as_ref().unwrap().resolve_request_body(reference)
                }
            };

            if body.is_some() {
                tabs.push(EndpointTab::Body);
            }
        }

        if !op.parameters.is_empty() {
            tabs.push(EndpointTab::Parameters);
        }

        if !op.responses.is_empty() {
            tabs.push(EndpointTab::Responses);
        }

        let tabs_count = tabs.len();

        if tabs_count == 0 {
            return;
        }

        let tab_bar = Tabs::new(
            tabs.iter()
                .enumerate()
                .map(|(i, t)| format!(" {} ({}) ", t.as_str(), i + 1)),
        )
        .block(Block::bordered())
        .select(app.endpoints_details_path_selected_tab_idx)
        .style(Style::default().white())
        .highlight_style(Style::default().white().bg(Color::Blue))
        .divider(symbols::DOT)
        .padding(" ", " ");

        f.render_widget(tab_bar, layout_1_v[0]);

        if let Some(content_type) = tabs.get(app.endpoints_details_path_selected_tab_idx) {
            match *content_type {
                EndpointTab::Body => {
                    render_body(f, app, &op, layout_1_v[1]);
                }
                EndpointTab::Parameters => {
                    render_parameter(f, app, &op, layout_1_v[1]);
                }
                EndpointTab::Responses => {
                    render_response(f, app, &op, layout_1_v[1]);
                }
            }

            app.endpoints_details_path_selected_tab = content_type.clone();
        }

        app.endpoints_details_path_tabs_count = tabs_count;
        if app.endpoints_details_path_selected_tab_idx >= tabs_count {
            app.endpoints_details_path_selected_tab_idx = 0;
        }
        // let responses_list = List::new(vec![ListItem::new("")]).block(Block::bordered());
        // f.render_widget(responses_list, layout_1_v[1]);
    } else {

        // Layout 2:
        // TODO: Related Components
    }

    // TODO: Add support to render Request Body, Parameters, Responses, and Components in full screen mode
}

fn render_body(f: &mut Frame, app: &mut App, op: &Operation, area: Rect) {
    let Some(spec) = app.spec.as_ref() else {
        return;
    };

    let body = op
        .request_body
        .as_ref()
        .expect("Request Body should be available here");

    let body = match body {
        RequestBodyOrRef::Item(body) => body,
        RequestBodyOrRef::Ref { reference } => app
            .spec
            .as_ref()
            .unwrap()
            .resolve_request_body(reference)
            .expect("Request Body struct should be available here"),
    };

    let items = body
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

            let mut lines = vec![Line::from(media_type.as_str())];

            if let Some(schema) = schema {
                let name = name.map_or_else(|| "Custom Schema".to_owned(), |t| t.to_owned());

                lines.extend(components::schema::ui(spec, schema, name, 2, true));
            } else if let Some(reference) = reference {
                lines.push(Line::from(Span::from(format!(
                    "{}{media_type}: {reference}",
                    " ".repeat(2)
                ))));
            } else {
                lines.push(Line::from(Span::from(format!(
                    "{}{media_type}: <unknown>",
                    " ".repeat(2)
                ))));
            }

            lines
        })
        .collect::<Vec<_>>();

    let select_item_index = app
        .endpoints_details_body_scroll_pos
        .checked_add(1)
        .unwrap_or(1);
    let items_len = items.len();

    let content = Paragraph::new(Text::from(items))
        .block(
            if app.endpoints_details_focused {
                Block::bordered().border_style(Style::default().blue())
            } else {
                Block::bordered()
            }
            .title_bottom(
                Line::from(format!("{}/{}", select_item_index, items_len)).right_aligned(),
            ),
        )
        .wrap(Wrap { trim: false });

    let lines = content.line_count(area.width);
    let scroll_pos = app
        .endpoints_details_body_scroll_pos
        .clamp(0, u16::try_from(items_len).unwrap_or(u16::MAX));

    let content = content.scroll((scroll_pos, 0));

    f.render_widget(content, area);

    // Add scrollbar
    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"));
    app.endpoints_details_body_scroll_pos = scroll_pos;
    let mut scroll_bar_state =
        ScrollbarState::new(lines).position(app.endpoints_details_body_scroll_pos as usize);

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

fn render_parameter(f: &mut Frame, app: &mut App, op: &Operation, area: Rect) {
    let Some(spec) = app.spec.as_ref() else {
        return;
    };

    let items = op
        .parameters
        .iter()
        .flat_map(|param| {
            let (param, reference) = match param {
                ParameterOrRef::Item(param) => (Some(param), None),
                ParameterOrRef::Ref { reference } => {
                    (spec.resolve_parameter(reference), Some(reference))
                }
            };

            if let Some(param) = param {
                let mut lines =
                    components::parameter::ui(spec, &param, param.name.clone(), 0, true);
                if !lines.is_empty() {
                    lines.push(Line::from(""));
                }

                lines
            } else if let Some(reference) = reference {
                vec![Line::from(Span::from(format!(
                    "{}Unknown: {reference}",
                    " ".repeat(2)
                )))]
            } else {
                vec![Line::from(Span::from(format!(
                    "{}Unknown: <unknown>",
                    " ".repeat(2)
                )))]
            }
        })
        .collect::<Vec<_>>();

    let select_item_index = app
        .endpoints_details_parameters_scroll_pos
        .checked_add(1)
        .unwrap_or(1);
    let items_len = items.len();

    let content = Paragraph::new(Text::from(items))
        .block(
            if app.endpoints_details_focused {
                Block::bordered().border_style(Style::default().blue())
            } else {
                Block::bordered()
            }
            .title_bottom(
                Line::from(format!("{}/{}", select_item_index, items_len)).right_aligned(),
            ),
        )
        .wrap(Wrap { trim: false });

    let lines = content.line_count(area.width);
    let scroll_pos = app
        .endpoints_details_parameters_scroll_pos
        .clamp(0, u16::try_from(items_len).unwrap_or(u16::MAX));

    let content = content.scroll((scroll_pos, 0));

    f.render_widget(content, area);

    // Add scrollbar
    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"));
    app.endpoints_details_parameters_scroll_pos = scroll_pos;
    let mut scroll_bar_state =
        ScrollbarState::new(lines).position(app.endpoints_details_parameters_scroll_pos as usize);

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

fn render_response(f: &mut Frame, app: &mut App, op: &Operation, area: Rect) {
    let Some(spec) = app.spec.as_ref() else {
        return;
    };

    let items = op
        .responses
        .iter()
        .flat_map(|(name, response)| {
            let (response, reference) = match response {
                ResponseOrRef::Item(param) => (Some(param), None),
                ResponseOrRef::Ref { reference } => {
                    (spec.resolve_response(reference), Some(reference))
                }
            };

            if let Some(response) = response {
                let mut lines = components::response::ui(spec, response, name.clone(), 0, true);

                if !lines.is_empty() {
                    lines.push(Line::from(""));
                }

                lines
            } else if let Some(reference) = reference {
                vec![Line::from(Span::from(format!(
                    "{}Unknown: {reference}",
                    " ".repeat(2)
                )))]
            } else {
                vec![Line::from(Span::from(format!(
                    "{}Unknown: <unknown>",
                    " ".repeat(2)
                )))]
            }
        })
        .collect::<Vec<_>>();

    let select_item_index = app
        .endpoints_details_responses_scroll_pos
        .checked_add(1)
        .unwrap_or(1);
    let items_len = items.len();

    let content = Paragraph::new(Text::from(items))
        .block(
            if app.endpoints_details_focused {
                Block::bordered().border_style(Style::default().blue())
            } else {
                Block::bordered()
            }
            .title_bottom(
                Line::from(format!("{}/{}", select_item_index, items_len)).right_aligned(),
            ),
        )
        .wrap(Wrap { trim: false });

    let lines = content.line_count(area.width);
    let scroll_pos = app
        .endpoints_details_responses_scroll_pos
        .clamp(0, u16::try_from(items_len).unwrap_or(u16::MAX));

    let content = content.scroll((scroll_pos, 0));

    f.render_widget(content, area);

    // Add scrollbar
    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"));
    app.endpoints_details_responses_scroll_pos = scroll_pos;
    let mut scroll_bar_state =
        ScrollbarState::new(lines).position(app.endpoints_details_responses_scroll_pos as usize);

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
