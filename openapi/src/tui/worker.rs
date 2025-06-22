use std::thread;

use crossbeam_channel::{unbounded, Receiver};

use super::state::{WorkerCommands, WorkerResults};

pub fn start_worker(receiver: Receiver<WorkerCommands>) -> Receiver<WorkerResults> {
    let (tx, rx) = unbounded();

    thread::spawn(move || {
        loop {
            match receiver.recv() {
                Ok(WorkerCommands::ParseSpec(source)) => {
                    let _ = tx.send(WorkerResults::ParsingSpec(source.clone()));

                    let content = match std::fs::read_to_string(&source) {
                        Ok(content) => content,
                        Err(e) => {
                            let _ = tx.send(WorkerResults::Error(format!(
                                "Failed to read OpenAPI Spec file: {}",
                                e
                            )));
                            continue;
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
