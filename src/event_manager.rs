use crossterm::event::{self, Event, KeyEvent};
use tokio::task::JoinHandle;

pub fn spawn() -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            match event::read() {
                Ok(Event::Key(key)) => handle_key(key),
                Ok(Event::Resize(cols, rows)) => handle_resize(cols, rows),
                _ => {}
            }
        }
    })
}

fn handle_key(key: KeyEvent) {
    // tmp
    if key.code == event::KeyCode::Char('q') {
        crate::tui::disable();

        std::process::exit(0);
    }
}

fn handle_resize(cols: u16, rows: u16) {}
