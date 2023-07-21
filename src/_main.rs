use std::error::Error;
use std::{
    io::{self, Stdout},
    time::Duration,
};

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, widgets::Paragraph, Terminal};


// Idee: 
// - 1 oder mehrere Dateien auswählen
// - unit Namen festlegen
// - out path festelegen
// - XSD parsen und in interne Representation konvertieren
// - Delphi Code generieren
//   - union type zu variant record, außer ref types enthalten
//   - enumarations zu enums mit helper class
//   - für alle Klassen eine forward declaration
//   - FromXml and FromXmlRaw constructor
//   - ToXml and ToXmlRaw functions

fn main() -> Result<(), Box<dyn Error>> {
    let mut terminal = setup_terminal()?;
    run(&mut terminal).expect("Failed to run main loop");
    restore_terminal(&mut terminal).expect("Failed to restore terminal");
    Ok(())
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>, Box<dyn Error>> {
    let mut stdout = io::stdout();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen)?;
    Ok(Terminal::new(CrosstermBackend::new(stdout))?)
}

fn restore_terminal(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
) -> Result<(), Box<dyn Error>> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(terminal.show_cursor()?)
}

fn run(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<(), Box<dyn Error>> {
    Ok(loop {
        terminal.draw(|frame| {
            let greeting = Paragraph::new("Hello World!");
            frame.render_widget(greeting, frame.size());
        })?;

        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                if KeyCode::Char('q') == key.code {
                    break;
                }
            }
        }
    })
}
