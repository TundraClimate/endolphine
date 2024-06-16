use crate::{actions::Action, app::App};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

fn is_pending(app: &App) -> bool {
    if let Action::Pending = &app.action {
        true
    } else {
        false
    }
}

pub fn handle_keys(app: &mut App, event: KeyEvent) -> bool {
    match event.code {
        KeyCode::Char('q') => !is_pending(&app),
        KeyCode::Esc => {
            app.action = Action::None;
            app.dialog = None;
            app.selected.clear();
            false
        }
        KeyCode::Char('j') => {
            if !is_pending(&app) {
                if let KeyModifiers::SHIFT = event.modifiers {
                    app.action = Action::Next(10);
                } else {
                    app.action = Action::Next(1);
                }
            }
            false
        }
        KeyCode::Char('k') => {
            if !is_pending(&app) {
                if let KeyModifiers::SHIFT = event.modifiers {
                    app.action = Action::Previous(10);
                } else {
                    app.action = Action::Previous(1);
                }
            }
            false
        }
        KeyCode::Char('v') => {
            if !is_pending(&app) {
                app.selected.push(app.cursor);
            }
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

pub fn handle_action(app: &mut App) {
    let action = &app.action;
    match action {
        Action::Previous(i) => {
            let cursor = app.cursor;
            if cursor - i > 0 {
                app.cursor = cursor - i;
            } else {
                app.cursor = 0;
            }
        }
        Action::Next(i) => {
            let cursor = app.cursor;
            if cursor + i < app.files.len() {
                app.cursor = app.files.len() - 1;
            }
        }
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

pub fn auto_selector(app: &mut App) {
    if !app.selected.is_empty() && !app.selected.contains(&app.cursor) {
        app.selected.push(app.cursor);
    }
}
