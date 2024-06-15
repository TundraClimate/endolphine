use crossterm::event::{KeyCode, KeyEvent};

pub fn handle_keys(event: KeyEvent) -> bool {
    match event.code {
        KeyCode::Char('q') => true,
        KeyCode::Esc => false,
        _ => false,
    }
}
