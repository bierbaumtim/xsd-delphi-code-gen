use std::{path::PathBuf, time::Duration};

use crossbeam_channel::{select, tick, unbounded};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    Terminal,
};

use crate::tui::state::*;

mod state;
mod ui;
mod worker;

pub fn run(source: PathBuf) -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stderr = std::io::stderr();
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    let (tx_worker, rx_worker) = unbounded();

    let worker_result_recv = worker::start_worker(rx_worker);

    let mut app = App::new(source, tx_worker, worker_result_recv);
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

fn run_app<'a, B: Backend>(terminal: &mut Terminal<B>, app: &'a mut App) -> anyhow::Result<bool> {
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
                        app.state = State::Parsing(path.to_string_lossy().to_string());
                    }
                    Ok(WorkerResults::SpecParsed(spec)) => {
                        app.set_parsed(spec);
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
    if event::poll(Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Release {
                return Ok(false);
            }

            match key.code {
                event::KeyCode::Char('q') => match app.state {
                    State::Initial => {
                        let _ = app.worker_sender.send(WorkerCommands::Shutdown);
                        return Ok(true);
                    }
                    State::Parsing(_) | State::Parsed | State::Error(_) => {
                        app.reset();
                    }
                },
                event::KeyCode::Char('1') if app.state == State::Parsed => match app.selected_tab {
                    0 if app.endpoints_details_focused => {
                        app.endpoints_details_path_selected_tab_idx = 0;
                    }
                    _ => app.selected_tab = 0,
                },
                event::KeyCode::Char('2') if app.state == State::Parsed => match app.selected_tab {
                    0 if app.endpoints_details_focused => {
                        app.endpoints_details_path_selected_tab_idx = 1;
                    }
                    _ => {
                        app.selected_tab = 1;
                        if app.components_navigation_list_state.selected().is_none() {
                            app.components_navigation_list_state.select_first();
                        }
                        if app.components_list_state.selected().is_none() {
                            app.components_list_state.select_first();
                        }
                    }
                },
                event::KeyCode::Char('3') if app.state == State::Parsed => match app.selected_tab {
                    0 if app.endpoints_details_focused => {
                        app.endpoints_details_path_selected_tab_idx = 2;
                    }
                    _ => app.selected_tab = 2,
                },
                //     event::KeyCode::Char('4') if app.mode == Mode::DisplayingHealthReport => {
                //         app.selected_tab = 3
                //     }
                event::KeyCode::Down if app.state == State::Parsed => {
                    handle_scroll_down(app, false);
                }
                event::KeyCode::Up if app.state == State::Parsed => {
                    handle_scroll_up(app, false);
                }
                event::KeyCode::Right if app.state == State::Parsed => match app.selected_tab {
                    0 if app.endpoints_list_state.selected().is_some() => {
                        app.endpoints_details_focused = true;
                    }
                    1 => {
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
                    0 => app.endpoints_details_focused = false,
                    1 => {
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
                    if matches!(app.state, State::Initial) =>
                {
                    let _ = app
                        .worker_sender
                        .send(WorkerCommands::ParseSpec(app.source.clone()));
                }
                //     event::KeyCode::PageUp if app.mode == Mode::DisplayingHealthReport => {
                //         handle_scroll_up(app, true);
                //     }
                //     event::KeyCode::PageDown if app.mode == Mode::DisplayingHealthReport => {
                //         handle_scroll_down(app, true);
                //     }
                _ => (),
            }
        }
    }

    Ok(false)
}

fn handle_scroll_down(app: &mut App, scroll_page: bool) {
    match app.selected_tab {
        0 => {
            if app.endpoints_details_focused {
                match app.endpoints_details_path_selected_tab {
                    EndpointTab::Parameters => {
                        app.endpoints_details_parameters_list_state
                            .scroll_down_by(1);
                    }
                    EndpointTab::Body => {
                        app.endpoints_details_body_scroll_pos =
                            app.endpoints_details_body_scroll_pos.saturating_add(1);
                    }
                    EndpointTab::Responses => {
                        app.endpoints_details_responses_list_state.scroll_down_by(1);
                    }
                }
                // let scroll_by: u16 = if scroll_page {
                //     // app.unused_code_selected_item_types_list_viewport
                //     //     .try_into()
                //     //     .unwrap_or(1)
                //     1
                // } else {
                //     1
                // };

                // app.endpoints_list_state.scroll_down_by(scroll_by);
            } else {
                let current_idx = app.endpoints_list_state.selected();

                app.endpoints_list_state.scroll_down_by(1);

                if app.endpoints_list_state.selected() != current_idx {
                    app.endpoints_details_body_scroll_pos = 0;
                    app.endpoints_details_parameters_list_state.select(None);
                    app.endpoints_details_responses_list_state.select(None);
                }
            }
        }
        1 => {
            match app.components_focused_region {
                ComponentsRegion::Navigation => {
                    app.components_navigation_list_state.scroll_down_by(1);
                }
                ComponentsRegion::List => {
                    let current_idx = app.components_list_state.selected();

                    app.components_list_state.scroll_down_by(1);

                    if app.components_list_state.selected() != current_idx {
                        app.components_details_scroll_pos = 0;
                        // app.endpoints_details_body_list_state.select(None);
                        // app.endpoints_details_parameters_list_state.select(None);
                        // app.endpoints_details_responses_list_state.select(None);
                    }
                }
                ComponentsRegion::Details => {
                    app.components_details_scroll_pos =
                        app.components_details_scroll_pos.saturating_add(1);
                }
                _ => (),
            }
        }
        // 2 => {
        //     if app.is_depencies_dependents_focused {
        //         let scroll_by: u16 = if scroll_page {
        //             app.dependencies_selected_item_types_list_viewport
        //                 .try_into()
        //                 .unwrap_or(1)
        //         } else {
        //             1
        //         };

        //         app.dependencies_selected_item_types_list_state
        //             .scroll_down_by(scroll_by);
        //     } else {
        //         let scroll_by: u16 = if scroll_page {
        //             app.dependencies_list_viewport.try_into().unwrap_or(1)
        //         } else {
        //             1
        //         };

        //         let current_idx = app.dependencies_list_state.selected();

        //         app.dependencies_list_state.scroll_down_by(scroll_by);

        //         if app.dependencies_list_state.selected() != current_idx {
        //             app.dependencies_selected_item_types_list_state.select(None);
        //         }
        //     }
        // }
        // 3 => {
        //     let scroll_by: u16 = if scroll_page {
        //         app.package_build_order_list_viewport
        //             .try_into()
        //             .unwrap_or(1)
        //     } else {
        //         1
        //     };

        //     app.package_build_order_list_state.scroll_down_by(scroll_by);
        // }
        _ => (),
    }
}

fn handle_scroll_up(app: &mut App, scroll_page: bool) {
    match app.selected_tab {
        0 => {
            if app.endpoints_details_focused {
                match app.endpoints_details_path_selected_tab {
                    EndpointTab::Parameters => {
                        app.endpoints_details_parameters_list_state.scroll_up_by(1);
                    }
                    EndpointTab::Body => {
                        app.endpoints_details_body_scroll_pos =
                            app.endpoints_details_body_scroll_pos.saturating_sub(1);
                    }
                    EndpointTab::Responses => {
                        app.endpoints_details_responses_list_state.scroll_up_by(1);
                    }
                }
                // let scroll_by: u16 = if scroll_page {
                //     app.unused_code_selected_item_types_list_viewport
                //         .try_into()
                //         .unwrap_or(1)
                // } else {
                //     1
                // };

                // app.unused_code_selected_item_types_list_state
                //     .scroll_up_by(scroll_by);
            } else {
                let current_idx = app.endpoints_list_state.selected();

                app.endpoints_list_state.scroll_up_by(1);

                if app.endpoints_list_state.selected() != current_idx {
                    app.endpoints_details_body_scroll_pos = 0;
                    app.endpoints_details_parameters_list_state.select(None);
                    app.endpoints_details_responses_list_state.select(None);
                }
            }
        }
        1 => {
            match app.components_focused_region {
                ComponentsRegion::Navigation => {
                    app.components_navigation_list_state.scroll_up_by(1);
                }
                ComponentsRegion::List => {
                    let current_idx = app.components_list_state.selected();

                    app.components_list_state.scroll_up_by(1);

                    if app.components_list_state.selected() != current_idx {
                        app.components_details_scroll_pos = 0;
                        // app.endpoints_details_body_list_state.select(None);
                        // app.endpoints_details_parameters_list_state.select(None);
                        // app.endpoints_details_responses_list_state.select(None);
                    }
                }
                ComponentsRegion::Details => {
                    app.components_details_scroll_pos =
                        app.components_details_scroll_pos.saturating_sub(1);
                }
            }
        }
        // 2 => {
        //     if app.is_depencies_dependents_focused {
        //         let scroll_by: u16 = if scroll_page {
        //             app.dependencies_selected_item_types_list_viewport
        //                 .try_into()
        //                 .unwrap_or(1)
        //         } else {
        //             1
        //         };

        //         app.dependencies_selected_item_types_list_state
        //             .scroll_up_by(scroll_by);
        //     } else {
        //         let scroll_by: u16 = if scroll_page {
        //             app.dependencies_list_viewport.try_into().unwrap_or(1)
        //         } else {
        //             1
        //         };
        //         let current_idx = app.dependencies_list_state.selected();

        //         app.dependencies_list_state.scroll_up_by(scroll_by);

        //         if app.dependencies_list_state.selected() != current_idx {
        //             app.dependencies_selected_item_types_list_state.select(None);
        //         }
        //     }
        // }
        // 3 => {
        //     let scroll_by: u16 = if scroll_page {
        //         app.package_build_order_list_viewport
        //             .try_into()
        //             .unwrap_or(1)
        //     } else {
        //         1
        //     };

        //     app.package_build_order_list_state.scroll_up_by(scroll_by);
        // }
        _ => (),
    }
}
