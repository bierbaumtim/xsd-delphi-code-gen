use ratatui::{prelude::*, widgets::*};

use crate::{
    parser::types::*,
    tui::state::{App, EndpointTab},
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
        app.endpoints_details_body_list_state = ListState::default();
        app.endpoints_details_parameters_list_state = ListState::default();
        app.endpoints_details_responses_list_state = ListState::default();
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
    let Some((color, path, method, endpoint, op)) = app.get_endpoint_at(index) else {
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
        // TODO: Layout 1:
        // TODO: Top 25% = Description Paragraph + ScrollBar
        // TODO: Main 1 50% or 75% if focused = Request Body or Parameters
        // TODO: Main 2 50% or 75% if focused = Responses

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

        let tab_bar = Tabs::new(tabs.iter().map(|t| t.as_str()))
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

                    let items = body.content.iter().map(|(media_type, media)| {
                        let schema = media.schema.as_ref().map_or("No Schema", |s| match s {
                            SchemaOrRef::Item(schema) => schema
                                .title
                                .as_ref()
                                .map_or("Custom Schema", |t| t.as_str()),
                            SchemaOrRef::Ref { reference } => reference,
                        });

                        let text = Text::from(vec![
                            Line::from(media_type.as_str()),
                            Line::from(format!("Schema: {schema}")),
                        ]);

                        ListItem::new(text)
                    });

                    let select_item_index = app
                        .endpoints_details_body_list_state
                        .selected()
                        .map_or(0, |i| i.checked_add(1).unwrap_or(1));
                    let items_len = items.len();

                    let content = List::new(items)
                        .block(
                            if app.endpoints_details_focused {
                                Block::bordered().border_style(Style::default().blue())
                            } else {
                                Block::bordered()
                            }
                            .title_bottom(
                                Line::from(format!("{}/{}", select_item_index, items_len))
                                    .right_aligned(),
                            ),
                        )
                        .style(Style::default().white())
                        .highlight_style(Style::default().blue().bold())
                        .highlight_symbol(">>")
                        .repeat_highlight_symbol(false)
                        .direction(ListDirection::TopToBottom);

                    if app.endpoints_details_body_list_state.selected().is_none() {
                        app.endpoints_details_body_list_state.select_first();
                    }

                    f.render_stateful_widget(
                        content,
                        layout_1_v[1],
                        &mut app.endpoints_details_body_list_state,
                    );
                }
                EndpointTab::Parameters => {
                    let items = op
                        .parameters
                        .iter()
                        .filter_map(|param| {
                            let param = match param {
                                ParameterOrRef::Item(param) => Some(param),
                                ParameterOrRef::Ref { reference } => {
                                    app.spec.as_ref().unwrap().resolve_parameter(reference)
                                }
                            }?;

                            let text = Text::from(vec![
                                Line::from(vec![
                                    Span::styled(
                                        param.name.as_str(),
                                        if param.required {
                                            Style::default().bold().fg(Color::Red)
                                        } else {
                                            Style::default().bold()
                                        },
                                    ),
                                    Span::styled(
                                        format!(" ({})", param.in_.to_string().to_uppercase()),
                                        Style::default().fg(Color::DarkGray),
                                    ),
                                ]),
                                Line::from(format!(
                                    "{}",
                                    param.description.clone().unwrap_or_default()
                                )),
                                Line::from(""),
                            ]);

                            Some(ListItem::new(text))
                        })
                        .collect::<Vec<_>>();

                    let select_item_index = app
                        .endpoints_details_parameters_list_state
                        .selected()
                        .map_or(0, |i| i.checked_add(1).unwrap_or(1));
                    let items_len = items.len();

                    let parameters_list = List::new(items)
                        .block(
                            if app.endpoints_details_focused {
                                Block::bordered().border_style(Style::default().blue())
                            } else {
                                Block::bordered()
                            }
                            .title_bottom(
                                Line::from(format!("{}/{}", select_item_index, items_len))
                                    .right_aligned(),
                            ),
                        )
                        .style(Style::default().white())
                        .highlight_style(Style::default().blue().bold())
                        .highlight_symbol(">>")
                        .repeat_highlight_symbol(false)
                        .direction(ListDirection::TopToBottom);

                    if app
                        .endpoints_details_parameters_list_state
                        .selected()
                        .is_none()
                    {
                        app.endpoints_details_parameters_list_state.select_first();
                    }

                    f.render_stateful_widget(
                        parameters_list,
                        layout_1_v[1],
                        &mut app.endpoints_details_parameters_list_state,
                    );
                }
                EndpointTab::Responses => {
                    let items = op
                        .responses
                        .iter()
                        .filter_map(|(name, response)| {
                            let response = match response {
                                ResponseOrRef::Item(param) => Some(param),
                                ResponseOrRef::Ref { reference } => {
                                    app.spec.as_ref().unwrap().resolve_response(reference)
                                }
                            }?;

                            let text = Text::from(vec![
                                // Line::from(vec![
                                //     Span::styled(
                                //         param.name.as_str(),
                                //         if param.required {
                                //             Style::default().bold().fg(Color::Red)
                                //         } else {
                                //             Style::default().bold()
                                //         },
                                //     ),
                                //     Span::styled(
                                //         format!(" ({})", param.in_.to_string().to_uppercase()),
                                //         Style::default().fg(Color::DarkGray),
                                //     ),
                                // ]),
                                Line::from(response.description.as_str()),
                                Line::from(""),
                            ]);

                            Some(ListItem::new(text))
                        })
                        .collect::<Vec<_>>();

                    let select_item_index = app
                        .endpoints_details_responses_list_state
                        .selected()
                        .map_or(0, |i| i.checked_add(1).unwrap_or(1));
                    let items_len = items.len();

                    let response_list = List::new(items)
                        .block(
                            if app.endpoints_details_focused {
                                Block::bordered().border_style(Style::default().blue())
                            } else {
                                Block::bordered()
                            }
                            .title_bottom(
                                Line::from(format!("{}/{}", select_item_index, items_len))
                                    .right_aligned(),
                            ),
                        )
                        .style(Style::default().white())
                        .highlight_style(Style::default().blue().bold())
                        .highlight_symbol(">>")
                        .repeat_highlight_symbol(false)
                        .direction(ListDirection::TopToBottom);

                    if app
                        .endpoints_details_responses_list_state
                        .selected()
                        .is_none()
                    {
                        app.endpoints_details_responses_list_state.select_first();
                    }

                    f.render_stateful_widget(
                        response_list,
                        layout_1_v[1],
                        &mut app.endpoints_details_responses_list_state,
                    );
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
