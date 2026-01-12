use std::time::Duration;

use crossbeam_channel::{select, tick, unbounded};
use ratatui::{
    Terminal,
    backend::{Backend, CrosstermBackend},
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event},
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    },
};

use crate::tui::state::*;

mod state;
mod ui;
mod worker;

pub fn run(source: String) -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stderr = std::io::stderr();
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    let (tx_worker, rx_worker) = unbounded();

    let worker_result_recv = worker::start_worker(rx_worker);

    let mut app = App::new(source.into(), tx_worker, worker_result_recv);
    let _ = app
        .worker_sender
        .send(WorkerCommands::ParseSpec(app.source.clone()));

    let _res = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> anyhow::Result<bool> {
    let rx_ticker = tick(Duration::from_millis(100));

    loop {
        terminal.draw(|f| ui::ui(f, app))?;

        if handle_events(app)? {
            break;
        }

        select! {
            recv(rx_ticker) -> _ => (),
            recv(app.worker_receiver) -> msg => {
                match msg {
                    Ok(WorkerResults::ParsingSpec(path)) => {
                        app.state = State::Parsing(path.to_string());
                    }
                    Ok(WorkerResults::SpecParsed(spec)) => {
                        app.set_parsed(spec);
                    }
                    Ok(WorkerResults::GeneratedCode(client_code, model_code)) => {
                        app.generated_client_code = Some(client_code);
                        app.generated_models_code = Some(model_code);
                        app.selected_tab = BrowserTab::GeneratedCode;
                    }
                    Ok(WorkerResults::Error(err)) => {
                        app.state = state::State::Error(err);
                    }
                    Err(_) => (),
                }
            }
            default => (),
        }
    }

    Ok(false)
}

fn handle_events(app: &mut App) -> anyhow::Result<bool> {
    if event::poll(Duration::from_millis(50))?
        && let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Release {
                return Ok(false);
            }

            match key.code {
                event::KeyCode::Esc | event::KeyCode::Char('q') => match app.state {
                    State::Initial => {
                        let _ = app.worker_sender.send(WorkerCommands::Shutdown);
                        return Ok(true);
                    }
                    State::Parsing(_) | State::Parsed | State::Error(_) => {
                        app.reset();
                    }
                },
                event::KeyCode::Char('1') if app.state == State::Parsed => match app.selected_tab {
                    BrowserTab::Endpoints if app.endpoints_details_focused => {
                        app.endpoints_details_path_selected_tab_idx = 0;
                    }
                    _ => app.selected_tab = BrowserTab::Endpoints,
                },
                event::KeyCode::Char('2') if app.state == State::Parsed => match app.selected_tab {
                    BrowserTab::Endpoints if app.endpoints_details_focused => {
                        app.endpoints_details_path_selected_tab_idx = 1;
                    }
                    _ => {
                        app.selected_tab = BrowserTab::Components;
                        if app.components_navigation_list_state.selected().is_none() {
                            app.components_navigation_list_state.select_first();
                        }
                        if app.components_list_state.selected().is_none() {
                            app.components_list_state.select_first();
                        }
                    }
                },
                event::KeyCode::Char('3') if app.state == State::Parsed => match app.selected_tab {
                    BrowserTab::Endpoints if app.endpoints_details_focused => {
                        app.endpoints_details_path_selected_tab_idx = 2;
                    }
                    _ => app.selected_tab = BrowserTab::Details,
                },
                event::KeyCode::Char('4') if app.state == State::Parsed => {
                    app.selected_tab = BrowserTab::GeneratedCode
                }
                event::KeyCode::Down if app.state == State::Parsed => {
                    handle_scroll_down(app, false);
                }
                event::KeyCode::Up if app.state == State::Parsed => {
                    handle_scroll_up(app, false);
                }
                event::KeyCode::Right if app.state == State::Parsed => match app.selected_tab {
                    BrowserTab::Endpoints if app.endpoints_list_state.selected().is_some() => {
                        app.endpoints_details_focused = true;
                    }
                    BrowserTab::Components => {
                        app.components_focused_region = match app.components_focused_region {
                            ComponentsRegion::Navigation => ComponentsRegion::List,
                            ComponentsRegion::List => ComponentsRegion::Details,
                            r => r,
                        }
                    }
                    // 2 if app.dependencies_list_state.selected().is_some() => {
                    //     app.is_depencies_dependents_focused = true;
                    // }
                    _ => (),
                },
                event::KeyCode::Left if app.state == State::Parsed => match app.selected_tab {
                    BrowserTab::Endpoints => app.endpoints_details_focused = false,
                    BrowserTab::Components => {
                        app.components_focused_region = match app.components_focused_region {
                            ComponentsRegion::List => ComponentsRegion::Navigation,
                            ComponentsRegion::Details => ComponentsRegion::List,
                            r => r,
                        }
                    }
                    // 2 => app.is_depencies_dependents_focused = false,
                    _ => (),
                },
                event::KeyCode::Enter | event::KeyCode::Char('p')
                    if app.state == State::Initial =>
                {
                    let _ = app
                        .worker_sender
                        .send(WorkerCommands::ParseSpec(app.source.clone()));
                }
                event::KeyCode::Char('i') if app.state == State::Initial => {
                    // TODO: Show overlay to input OpenAPI spec URL or path
                }
                event::KeyCode::Char('g') if app.state == State::Parsed => {
                    if let Some(spec) = &app.spec {
                        let _ = app
                            .worker_sender
                            .send(WorkerCommands::GenerateCode(spec.clone()));
                    }
                }
                event::KeyCode::PageUp if app.state == State::Parsed => {
                    handle_scroll_up(app, true);
                }
                event::KeyCode::PageDown if app.state == State::Parsed => {
                    handle_scroll_down(app, true);
                }
                _ => (),
            }
        }

    Ok(false)
}

fn handle_scroll_down(app: &mut App, scroll_page: bool) {
    match app.selected_tab {
        BrowserTab::Endpoints => {
            if app.endpoints_details_focused {
                match app.endpoints_details_path_selected_tab {
                    EndpointTab::Parameters => {
                        app.endpoints_details_parameters_scroll_pos = app
                            .endpoints_details_parameters_scroll_pos
                            .saturating_add(1);
                    }
                    EndpointTab::Body => {
                        app.endpoints_details_body_scroll_pos =
                            app.endpoints_details_body_scroll_pos.saturating_add(1);
                    }
                    EndpointTab::Responses => {
                        app.endpoints_details_responses_scroll_pos =
                            app.endpoints_details_responses_scroll_pos.saturating_add(1);
                    }
                }
            } else {
                let current_idx = app.endpoints_list_state.selected();

                app.endpoints_list_state.scroll_down_by(1);

                if app.endpoints_list_state.selected() != current_idx {
                    app.endpoints_details_parameters_scroll_pos = 0;
                    app.endpoints_details_body_scroll_pos = 0;
                    app.endpoints_details_responses_scroll_pos = 0;
                }
            }
        }
        BrowserTab::Components => match app.components_focused_region {
            ComponentsRegion::Navigation => {
                app.components_navigation_list_state.scroll_down_by(1);
                app.components_list_state.select_first();
            }
            ComponentsRegion::List => {
                let current_idx = app.components_list_state.selected();

                app.components_list_state.scroll_down_by(1);

                if app.components_list_state.selected() != current_idx {
                    app.components_details_scroll_pos = 0;
                }
            }
            ComponentsRegion::Details => {
                app.components_details_scroll_pos =
                    app.components_details_scroll_pos.saturating_add(1);
            }
        },
        BrowserTab::Details => {
            let scroll_by: u16 = if scroll_page {
                app.details_viewport_height
            } else {
                1
            };
            app.details_scroll_pos = app.details_scroll_pos.saturating_add(scroll_by);
        }
        BrowserTab::GeneratedCode => {
            let scroll_by: u16 = if scroll_page {
                app.generated_code_viewport_height
            } else {
                1
            };
            app.generated_code_scroll_pos = app.generated_code_scroll_pos.saturating_add(scroll_by);
        }
    }
}

fn handle_scroll_up(app: &mut App, scroll_page: bool) {
    match app.selected_tab {
        BrowserTab::Endpoints => {
            if app.endpoints_details_focused {
                match app.endpoints_details_path_selected_tab {
                    EndpointTab::Parameters => {
                        app.endpoints_details_parameters_scroll_pos = app
                            .endpoints_details_parameters_scroll_pos
                            .saturating_sub(1);
                    }
                    EndpointTab::Body => {
                        app.endpoints_details_body_scroll_pos =
                            app.endpoints_details_body_scroll_pos.saturating_sub(1);
                    }
                    EndpointTab::Responses => {
                        app.endpoints_details_responses_scroll_pos =
                            app.endpoints_details_responses_scroll_pos.saturating_sub(1);
                    }
                }
            } else {
                let current_idx = app.endpoints_list_state.selected();

                app.endpoints_list_state.scroll_up_by(1);

                if app.endpoints_list_state.selected() != current_idx {
                    app.endpoints_details_parameters_scroll_pos = 0;
                    app.endpoints_details_body_scroll_pos = 0;
                    app.endpoints_details_responses_scroll_pos = 0;
                }
            }
        }
        BrowserTab::Components => match app.components_focused_region {
            ComponentsRegion::Navigation => {
                app.components_navigation_list_state.scroll_up_by(1);
                app.components_list_state.select_first();
            }
            ComponentsRegion::List => {
                let current_idx = app.components_list_state.selected();

                app.components_list_state.scroll_up_by(1);

                if app.components_list_state.selected() != current_idx {
                    app.components_details_scroll_pos = 0;
                }
            }
            ComponentsRegion::Details => {
                app.components_details_scroll_pos =
                    app.components_details_scroll_pos.saturating_sub(1);
            }
        },
        BrowserTab::Details => {
            let scroll_by: u16 = if scroll_page {
                app.details_viewport_height
            } else {
                1
            };

            app.details_scroll_pos = app.details_scroll_pos.saturating_sub(scroll_by);
        }
        BrowserTab::GeneratedCode => {
            let scroll_by: u16 = if scroll_page {
                app.generated_code_viewport_height
            } else {
                1
            };
            app.generated_code_scroll_pos = app.generated_code_scroll_pos.saturating_sub(scroll_by);
        }
    }
}
