use crate::{
    actions::Action,
    app::App,
    shell,
    ui::{self, Dialog},
};
use crossterm::{
    event::{Event, KeyCode, KeyEvent},
    execute,
};
use std::io;
use tui_input::backend::crossterm::EventHandler;

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
            execute!(
                io::stdout(),
                crossterm::terminal::Clear(crossterm::terminal::ClearType::CurrentLine)
            )
            .unwrap();
            false
        }
        KeyCode::Char('j') => {
            if !is_pending(&app) {
                app.action = Action::Next(1);
            }
            false
        }
        KeyCode::Char('J') => {
            if !is_pending(&app) {
                app.action = Action::Next(10);
            }
            false
        }
        KeyCode::Char('k') => {
            if !is_pending(&app) {
                app.action = Action::Previous(1);
            }
            false
        }
        KeyCode::Char('K') => {
            if !is_pending(&app) {
                app.action = Action::Previous(10);
            }
            false
        }
        KeyCode::Char('v') => {
            if !is_pending(&app) {
                app.selected.push(app.cursor);
            }
            false
        }
        KeyCode::Char('c') => {
            if !is_pending(&app) {
                app.action = Action::Cut;
            }
            false
        }
        KeyCode::Char('y') => {
            if !is_pending(&app) {
                app.action = Action::Copy;
            }
            false
        }
        KeyCode::Char('p') => {
            if !is_pending(&app) {
                let register = &mut app.register;
                let current_dir = &app.path;
                if app.is_cut {
                    register.iter().for_each(|p| {
                        shell::mv(p.clone(), current_dir.clone());
                    });
                    register.clear();
                    app.is_cut = false;
                } else {
                    register.iter().for_each(|p| {
                        shell::cp(p.clone(), current_dir.clone());
                    });
                }
            }
            false
        }
        KeyCode::Char('a') => {
            if !is_pending(&app) {
                app.action = Action::Create;
            }
            false
        }
        KeyCode::Enter => {
            if is_pending(&app) {
                if let Some(dialog) = &app.dialog {
                    if dialog.input.value().is_empty() {
                        app.action = Action::None;
                        app.dialog = None;
                    } else {
                        app.action = Action::Confirm;
                    }
                    execute!(
                        io::stdout(),
                        crossterm::terminal::Clear(crossterm::terminal::ClearType::CurrentLine)
                    )
                    .unwrap();
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
            if cursor >= *i {
                app.cursor = cursor - i;
            } else {
                app.cursor = 0;
            }
            app.action = Action::None;
        }
        Action::Next(i) => {
            let cursor = app.cursor;
            if cursor + i < app.files.len() {
                app.cursor = app.files.len() - 1;
            }
            app.action = Action::None;
        }
        Action::Create => {
            app.dialog = Some(Dialog {
                action: Action::Create,
                input: "".into(),
            });
            if let Some(ref dialog) = app.dialog {
                ui::write_backend(dialog, "New file/directory:").unwrap();
            }
            app.action = Action::Pending;
        }
        Action::Delete(path) => {}
        Action::Cut => {
            app.is_cut = true;
            app.action = Action::Copy;
        }
        Action::Copy => {
            if app.selected.is_empty() {
                let file = app.files[app.cursor].clone();
                app.register.push(file);
            } else {
                app.selected
                    .iter()
                    .for_each(|i| app.register.push(app.files[*i].clone()));
            }
            app.action = Action::None;
        }
        Action::Rename(path) => {}
        Action::Pending => {}
        Action::Confirm => {
            if let Some(Dialog { action, input }) = &app.dialog {
                let value = input.value();
                match action {
                    Action::Create => {
                        if let Some(suff) = value.chars().last() {
                            if suff == '/' {
                                shell::mkdir(app.path.join(value));
                            } else {
                                shell::create(app.path.join(value));
                            }
                        }
                    }
                    _ => {}
                }
            }
            app.action = Action::None;
        }
        Action::None => {}
    }
}

pub fn handle_dialog(app: &mut App, event: &Event) {
    if is_pending(&app) {
        if let Some(ref mut dialog) = app.dialog {
            let text = match dialog.action {
                Action::Create => "New file/directory:",
                _ => "",
            };
            if dialog.input.handle_event(&event).is_some() {
                ui::write_backend(&dialog, text).unwrap();
            }
        }
    }
}

pub fn auto_selector(app: &mut App) {
    if !app.selected.is_empty() && !app.selected.contains(&app.cursor) {
        app.selected.push(app.cursor);
    }
}
