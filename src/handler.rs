use crate::{actions::Action, app::App};
use crossterm::event::{KeyCode, KeyEvent};

pub fn handle_keys(app: &mut App, event: KeyEvent) -> bool {
    match event.code {
        KeyCode::Char('q') => {
            if let Action::Pending = &app.action {
                false
            } else {
                true
            }
        }
        KeyCode::Esc => {
            app.action = Action::None;
            app.dialog = None;
            false
        }
        KeyCode::Enter => {
            if let Action::Pending = &app.action {
                if let Some(dialog) = &app.dialog {
                    if dialog.input.is_empty() {
                        app.action = Action::None;
                        app.dialog = None;
                        return false;
                    } else {
                        app.action = Action::Confirm;
                    }
                }
            }
            false
        }
        _ => false,
    }
}

pub fn handle_action(app: &mut App, action: Action) {
    match action {
        Action::Previous(i) => {}
        Action::Next(i) => {}
        Action::Create(ctype) => {}
        Action::Delete(path) => {}
        Action::Cut(from) => {}
        Action::Copy(from) => {}
        Action::Rename(path) => {}
        Action::Pending => {}
        Action::Confirm => {}
        Action::None => {}
    }
}
