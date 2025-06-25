use std::thread;

use crossbeam_channel::{unbounded, Receiver};

use crate::tui::state::Source;

use super::state::{WorkerCommands, WorkerResults};

pub fn start_worker(receiver: Receiver<WorkerCommands>) -> Receiver<WorkerResults> {
    let (tx, rx) = unbounded();

    thread::spawn(move || {
        loop {
            match receiver.recv() {
                Ok(WorkerCommands::ParseSpec(source)) => {
                    let _ = tx.send(WorkerResults::ParsingSpec(source.clone()));

                    let content = match source {
                        Source::File(path_buf) => match std::fs::read_to_string(&path_buf) {
                            Ok(content) => content,
                            Err(e) => {
                                let _ = tx.send(WorkerResults::Error(format!(
                                    "Failed to read OpenAPI Spec file: {}",
                                    e
                                )));
                                continue;
                            }
                        },
                        Source::Url(url) => {
                            let res = reqwest::blocking::get(url)
                                .map_err(|e| {
                                    WorkerResults::Error(format!(
                                        "Failed to fetch OpenAPI Spec from URL: {}",
                                        e
                                    ))
                                })
                                .and_then(|res| {
                                    res.text().map_err(|e| {
                                        WorkerResults::Error(format!(
                                            "Failed to read OpenAPI Spec from URL: {}",
                                            e
                                        ))
                                    })
                                });

                            match res {
                                Ok(content) => content,
                                Err(e) => {
                                    let _ = tx.send(e);
                                    continue;
                                }
                            }
                        }
                    };

                    // Collect health data
                    let spec = match crate::parser::from_json_str(&content) {
                        Ok(spec) => spec,
                        Err(e) => {
                            let _ = tx.send(WorkerResults::Error(format!(
                                "Failed to parse OpenAPI Spec: {}",
                                e
                            )));
                            continue;
                        }
                    };

                    // Send the health report back to the UI
                    let _ = tx.send(WorkerResults::SpecParsed(spec));
                }
                Ok(WorkerCommands::Shutdown) => break,
                Err(_) => (),
            }
        }
    });

    rx
}
