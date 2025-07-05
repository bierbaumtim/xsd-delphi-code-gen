use ratatui::{prelude::*, widgets::*};

use crate::tui::state::{App, BrowserTab};

mod components;
mod endpoints;

pub fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::new(
        Direction::Vertical,
        [
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ],
    )
    .split(f.area());

    let top_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Min(0), Constraint::Max(40)])
        .split(chunks[0]);

    let version_text = app
        .spec
        .as_ref()
        .map_or_else(|| "  -  ".to_string(), |spec| spec.info.version.clone());

    let paragraph = Paragraph::new(Line::from(vec![
        Span::from("Version: "),
        Span::styled(version_text, Style::default().fg(Color::DarkGray)),
    ]))
    .block(Block::bordered());
    f.render_widget(paragraph, top_layout[1]);

    let tab_bar = Tabs::new(vec![
        " Endpoints (1) ",
        " Components (2) ",
        " Details (3) ",
        " Generated Code (4) ",
    ])
    .block(Block::bordered().title("GenPhi - OpenAPI"))
    .select(app.selected_tab.index())
    .style(Style::default().white())
    .highlight_style(Style::default().white().bg(Color::Blue))
    .divider(symbols::DOT)
    .padding(" ", " ");

    f.render_widget(tab_bar, top_layout[0]);

    match app.selected_tab {
        BrowserTab::Endpoints => endpoints::ui(f, app, chunks[1]),
        BrowserTab::Components => components::ui(f, app, chunks[1]),
        BrowserTab::Details => render_details(f, app, chunks[1]),
        BrowserTab::GeneratedCode => render_code(f, app, chunks[1]),
    }
}

fn render_details(f: &mut Frame, app: &mut App, area: Rect) {
    let Some(spec) = &app.spec else {
        return;
    };

    let mut lines = vec![
        Line::from(vec![
            Span::from("Title: "),
            Span::styled(
                spec.info.title.clone(),
                Style::default().fg(Color::DarkGray),
            ),
        ]),
        Line::from(vec![
            Span::from("Version: "),
            Span::styled(
                spec.info.version.as_str(),
                Style::default().fg(Color::DarkGray),
            ),
        ]),
    ];

    if let Some(contact) = &spec.info.contact {
        lines.push(Line::from(vec![Span::from("Contact: ")]));

        if let Some(name) = &contact.name {
            lines.push(Line::from(vec![
                Span::from("  Name: "),
                Span::styled(name.clone(), Style::default().fg(Color::DarkGray)),
            ]));
        }

        if let Some(email) = &contact.email {
            lines.push(Line::from(vec![
                Span::from("  Email: "),
                Span::styled(email.clone(), Style::default().fg(Color::DarkGray)),
            ]));
        }
        if let Some(url) = &contact.url {
            lines.push(Line::from(vec![
                Span::from("  URL: "),
                Span::styled(url.clone(), Style::default().fg(Color::DarkGray)),
            ]));
        }

        lines.push(Line::from(""));
    }

    if let Some(description) = &spec.info.description {
        lines.push(Line::from(vec![
            Span::from("Description: "),
            Span::styled(description.clone(), Style::default().fg(Color::DarkGray)),
        ]));
    }

    if let Some(terms_of_service) = &spec.info.terms_of_service {
        lines.push(Line::from(vec![
            Span::from("Terms of Service: "),
            Span::styled(
                terms_of_service.clone(),
                Style::default().fg(Color::DarkGray),
            ),
        ]));
    }

    let text = Text::from(lines);

    let paragraph = Paragraph::new(text)
        .block(Block::bordered().border_style(Style::default().blue()))
        .wrap(Wrap { trim: false });

    let lines = paragraph.line_count(area.width);
    let scroll_pos = app.details_scroll_pos.clamp(
        0,
        u16::try_from(lines)
            .unwrap_or(u16::MAX)
            .saturating_sub(area.height),
    );

    let paragraph = paragraph.scroll((scroll_pos, 0));

    f.render_widget(paragraph, area);

    // Add scrollbar
    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"));
    app.details_scroll_pos = scroll_pos;
    let mut scroll_bar_state = ScrollbarState::new(lines).position(app.details_scroll_pos as usize);

    f.render_stateful_widget(
        scrollbar,
        area.inner(Margin {
            // using an inner vertical margin of 1 unit makes the scrollbar inside the block
            vertical: 1,
            horizontal: 0,
        }),
        &mut scroll_bar_state,
    );

    app.details_viewport_height = area.height;
}

fn render_code(f: &mut Frame, app: &mut App, area: Rect) {
    let text = if let Some(code) = app.generated_models_code.as_ref() {
        Text::from(code.as_str())
    } else {
        Text::from("No code generated")
    };

    let paragraph = Paragraph::new(text)
        .block(Block::bordered().border_style(Style::default().blue()))
        .wrap(Wrap { trim: false });

    let lines = paragraph.line_count(area.width);
    let scroll_pos = app.generated_code_scroll_pos.clamp(
        0,
        u16::try_from(lines)
            .unwrap_or(u16::MAX)
            .saturating_sub(area.height),
    );

    let paragraph = paragraph.scroll((scroll_pos, 0));

    f.render_widget(paragraph, area);

    // Add scrollbar
    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"));
    app.generated_code_scroll_pos = scroll_pos;
    let mut scroll_bar_state =
        ScrollbarState::new(lines).position(app.generated_code_scroll_pos as usize);

    f.render_stateful_widget(
        scrollbar,
        area.inner(Margin {
            // using an inner vertical margin of 1 unit makes the scrollbar inside the block
            vertical: 1,
            horizontal: 0,
        }),
        &mut scroll_bar_state,
    );

    app.generated_code_viewport_height = area.height;
}
