use crossterm::{
    event::{KeyCode, KeyEvent, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, prelude::*, terminal::Terminal};
use std::{error::Error, io};

pub async fn render_mode() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    let (mut rc, shatdown) = crossterm_keyreader::spawn();

    loop {
        terminal.draw(ui)?;
        if let Ok(event) = rc.try_recv() {
            if event.kind == KeyEventKind::Press && handle_keys(event) {
                break;
            }
        }
    }

    shatdown.send(()).ok();
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}

fn handle_keys(event: KeyEvent) -> bool {
    match event.code {
        KeyCode::Char('q') => true,
        _ => false,
    }
}

fn ui(frame: &mut Frame) {}
