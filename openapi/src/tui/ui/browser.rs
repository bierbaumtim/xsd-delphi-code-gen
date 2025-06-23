use ratatui::{prelude::*, widgets::*};

use crate::tui::state::App;

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

    let tab_bar = Tabs::new(vec![" Endpoints ", " Components ", " Details "])
        .block(Block::bordered().title("GenPhi - OpenAPI"))
        .select(app.selected_tab)
        .style(Style::default().white())
        .highlight_style(Style::default().white().bg(Color::Blue))
        .divider(symbols::DOT)
        .padding(" ", " ");

    f.render_widget(tab_bar, top_layout[0]);

    match app.selected_tab {
        0 => endpoints::ui(f, app, chunks[1]),
        1 => components::ui(f, app, chunks[1]),
        2 => render_details(f, app, chunks[1]),
        _ => {}
    }
}

fn render_details(f: &mut Frame, app: &mut App, area: Rect) {
    let text = Text::from(vec![
        "Details".into(),
        "".into(),
        "This section can be used to display detailed information about the selected endpoint or component.".into(),
    ])
    .centered();

    let paragraph = Paragraph::new(text).block(Block::bordered()).centered();
    f.render_widget(paragraph, area);
}
