use crate::{actions::Action, app::App};
use crossterm::event::{KeyCode, KeyEvent};

pub fn handle_keys(app: &mut App, event: KeyEvent) -> bool {
    match event.code {
        KeyCode::Char('q') => true,
        KeyCode::Esc => {
            app.action = Action::None;
            false
        }
        _ => false,
    }
}
