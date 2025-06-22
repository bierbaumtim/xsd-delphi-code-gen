use ratatui::{prelude::*, widgets::*};

use crate::tui::state::{App, State};

mod browser;

pub fn ui(f: &mut Frame, app: &mut App) {
    match app.state {
        State::Initial => render_menu(f, app),
        State::Parsing(ref msg) => render_parsing(f, msg),
        State::Parsed => browser::ui(f, app),
        State::Error(ref err) => render_error(f, err),
    }
}

fn render_menu(f: &mut Frame, _app: &App) {
    let text = Text::from(vec![
        "Welcome to GenPhi - OpenAPI!".into(),
        "".into(),
        "".into(),
        Line::from(vec![
            "Version: ".white(),
            env!("CARGO_PKG_VERSION").dark_gray(),
        ]),
        Line::from(vec![
            "Author: ".white(),
            env!("CARGO_PKG_AUTHORS").dark_gray(),
        ]),
        Line::from(vec![
            "Repository: ".white(),
            env!("CARGO_PKG_REPOSITORY").dark_gray(),
        ]),
        "".into(),
        "".into(),
        "Press enter or 'p' to start".into(),
        "Press 'q' to quit".into(),
    ])
    .centered();
    let paragraph = Paragraph::new(text).block(Block::bordered()).centered();

    f.render_widget(paragraph, f.area());
}

fn render_parsing(f: &mut Frame, msg: &str) {
    let text = Text::from(vec![
        "Parsing OpenAPI Spec ...".into(),
        "".into(),
        msg.gray().into(),
    ])
    .centered();

    let paragraph = Paragraph::new(text).block(Block::bordered()).centered();

    f.render_widget(paragraph, f.area());
}

fn render_error(f: &mut Frame, error: &str) {
    let text = Text::from(vec![
        "An error occurred while processing the OpenAPI Spec:".into(),
        "".into(),
        error.red().into(),
    ]);
    let paragraph = Paragraph::new(text).block(Block::bordered()).centered();

    f.render_widget(paragraph, f.area());
}
