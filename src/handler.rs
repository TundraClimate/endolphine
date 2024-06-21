use crate::{
    actions::Action,
    app::App,
    shell,
    ui::{self, Dialog},
};
use crossterm::{
    cursor::MoveTo,
    event::{Event, KeyCode, KeyEvent},
    execute,
    terminal::{self, Clear, ClearType},
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
            let (cols, _) = terminal::size().unwrap();
            execute!(io::stdout(), MoveTo(0, cols)).unwrap();
            execute!(io::stdout(), Clear(ClearType::CurrentLine)).unwrap();
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
                        if let Some(parent) = p.parent() {
                            if &parent.to_path_buf() != current_dir {
                                shell::mv(p.clone(), current_dir.clone());
                            }
                        }
                    });
                    register.clear();
                    app.is_cut = false;
                } else {
                    register.iter().for_each(|p| {
                        if let Some(parent) = p.parent() {
                            if &parent.to_path_buf() != current_dir {
                                shell::cp(p.clone(), current_dir.clone());
                            }
                        }
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
        KeyCode::Char('d') => {
            if !is_pending(&app) {
                app.action = Action::Delete;
            }
            false
        }
        KeyCode::Char('r') => {
            if !is_pending(&app) {
                app.action = Action::Rename;
            }
            false
        }
        KeyCode::Char('h') => {
            if !is_pending(&app) {
                app.action = Action::Back;
            }
            false
        }
        KeyCode::Char('l') => {
            if !is_pending(&app) {
                app.action = Action::Open;
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
                    let (cols, _) = terminal::size().unwrap();
                    execute!(io::stdout(), MoveTo(0, cols)).unwrap();
                    execute!(io::stdout(), Clear(ClearType::CurrentLine)).unwrap();
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
                app.cursor = cursor + i;
            } else {
                app.cursor = app.files.len() - 1;
            }
            app.action = Action::None;
        }
        Action::Back => {
            if let Some(parent) = app.path.parent() {
                app.path = parent.to_path_buf();
                app.cursor = 0;
                app.selected.clear();
            }
            app.action = Action::None;
        }
        Action::Open => {
            let cur_position = &app.files[app.cursor];
            if cur_position.is_dir() {
                app.path = cur_position.clone();
                app.cursor = 0;
                app.selected.clear();
            } else {
                use std::io::Read;
                let mut file = std::fs::File::open(cur_position).unwrap();
                let mut buffer = [0; 1024];
                let read = file.read(&mut buffer).unwrap();
                if std::str::from_utf8(&buffer[..read]).is_ok() {}
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
        Action::Delete => {
            app.dialog = Some(Dialog {
                action: Action::Delete,
                input: "".into(),
            });
            if let Some(ref dialog) = app.dialog {
                if app.selected.is_empty() {
                    ui::write_backend(
                        dialog,
                        format!(
                            "Delete \"{}\" ? (y/N)",
                            crate::filename(&app.files[app.cursor])
                        )
                        .as_str(),
                    )
                    .unwrap();
                } else {
                    let len = app.selected.len();
                    ui::write_backend(dialog, format!("Delete {} items? (y/N)", len).as_str())
                        .unwrap();
                }
            }
            app.action = Action::Pending;
        }
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
        Action::Rename => {
            app.dialog = Some(Dialog {
                action: Action::Rename,
                input: crate::filename(&app.files[app.cursor]).into(),
            });
            if let Some(ref dialog) = app.dialog {
                ui::write_backend(
                    dialog,
                    format!("Rename \"{}\" :", crate::filename(&app.files[app.cursor])).as_str(),
                )
                .unwrap();
            }
            app.action = Action::Pending;
        }
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
                    Action::Delete => {
                        if value == "y" || value == "Y" {
                            if app.selected.is_empty() {
                                shell::trash_file(app.files[app.cursor].clone());
                            } else {
                                app.selected
                                    .iter()
                                    .for_each(|i| shell::trash_file(app.files[*i].clone()));
                            }
                        }
                    }
                    Action::Rename => {
                        if crate::filename(&app.files[app.cursor]) != value {
                            shell::mv(app.files[app.cursor].clone(), app.path.join(value));
                        }
                    }
                    _ => {}
                }
            }
            app.dialog = None;
            app.action = Action::None;
        }
        Action::None => {}
    }
}

pub fn handle_dialog(app: &mut App, event: &Event) {
    if is_pending(&app) {
        if let Some(ref mut dialog) = app.dialog {
            let text = match dialog.action {
                Action::Create => "New file/directory:".to_string(),
                Action::Delete => {
                    if app.selected.is_empty() {
                        format!(
                            "Delete \"{}\" ? (y/N)",
                            crate::filename(&app.files[app.cursor])
                        )
                    } else {
                        let len = app.selected.len();
                        format!("Delete {} items? (y/N)", len)
                    }
                }
                Action::Rename => {
                    format!("Rename \"{}\" :", crate::filename(&app.files[app.cursor]))
                }
                _ => String::new(),
            };
            if dialog.input.handle_event(&event).is_some() {
                ui::write_backend(&dialog, text.as_str()).unwrap();
            }
        }
    }
}

pub fn auto_selector(app: &mut App) {
    if !app.selected.is_empty() {
        if app.selected[0] <= app.cursor {
            app.selected = (app.selected[0]..=app.cursor).collect();
        } else {
            app.selected = (app.cursor..=app.selected[0]).rev().collect();
        }
    }
}
