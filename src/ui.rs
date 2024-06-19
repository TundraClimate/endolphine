use crate::actions::Action;
use crossterm::{
    cursor::MoveTo,
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor},
    terminal::{
        self, disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use ratatui::{backend::CrosstermBackend, prelude::*, terminal::Terminal};
use std::{error::Error, io};
use tui_input::{backend::crossterm as backend, Input};

pub fn render_mode<F: FnMut() -> bool>(mut looper: F) -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    loop {
        terminal.draw(ui)?;
        if looper() {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}

fn ui(frame: &mut Frame) {
    let (cols, rows) = terminal::size().unwrap();
    execute!(io::stdout(), MoveTo(0, rows - 2)).unwrap();
    execute!(io::stdout(), SetBackgroundColor(Color::Grey)).unwrap();
    execute!(io::stdout(), Clear(ClearType::CurrentLine)).unwrap();
    execute!(io::stdout(), Print(" ".repeat(cols as usize))).unwrap();
    execute!(io::stdout(), ResetColor).unwrap();
}

pub struct Dialog {
    pub input: Input,
    pub action: Action,
}

pub fn write_backend(dialog: &Dialog, text: &str) -> io::Result<()> {
    execute!(io::stdout(), MoveTo(0, 40)).unwrap();
    execute!(io::stdout(), Print(text)).unwrap();
    backend::write(
        &mut io::stdout(),
        dialog.input.value(),
        dialog.input.cursor(),
        ((text.len() + 1) as u16, 40),
        30,
    )
}
