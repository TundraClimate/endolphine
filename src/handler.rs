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

impl App {
    pub fn handle_keys(&mut self, event: KeyEvent) {
        match event.code {
            KeyCode::Char('Q') => self.quit = !is_pending(&self),
            KeyCode::Esc => {
                self.action = Action::Clean;
            }
            KeyCode::Char('j') => {
                if !is_pending(&self) {
                    self.action = Action::Next(1);
                }
            }
            KeyCode::Char('J') => {
                if !is_pending(&self) {
                    self.action = Action::Next(10);
                }
            }
            KeyCode::Char('k') => {
                if !is_pending(&self) {
                    self.action = Action::Previous(1);
                }
            }
            KeyCode::Char('K') => {
                if !is_pending(&self) {
                    self.action = Action::Previous(10);
                }
            }
            KeyCode::Char('v') => {
                if !is_pending(&self) {
                    self.selected.push(self.cursor);
                }
            }
            KeyCode::Char('c') => {
                if !is_pending(&self) {
                    self.action = Action::Cut;
                }
            }
            KeyCode::Char('y') => {
                if !is_pending(&self) {
                    self.action = Action::Copy;
                }
            }
            KeyCode::Char('p') => {
                if !is_pending(&self) {
                    self.action = Action::Paste;
                }
            }
            KeyCode::Char('a') => {
                if !is_pending(&self) {
                    self.action = Action::Create;
                }
            }
            KeyCode::Char('d') => {
                if !is_pending(&self) {
                    self.action = Action::Delete;
                }
            }
            KeyCode::Char('r') => {
                if !is_pending(&self) {
                    self.action = Action::Rename;
                }
            }
            KeyCode::Char('h') => {
                if !is_pending(&self) {
                    self.action = Action::Back;
                }
            }
            KeyCode::Char('l') => {
                if !is_pending(&self) {
                    self.action = Action::Open;
                }
            }
            KeyCode::Enter => {
                if is_pending(&self) {
                    self.action = Action::PreConfirm;
                }
            }
            _ => {}
        }
    }

    pub fn handle_action(&mut self) {
        let action = &self.action;
        match action {
            Action::Previous(i) => {
                let cursor = self.cursor;
                if cursor >= *i {
                    self.cursor = cursor - i;
                } else {
                    self.cursor = 0;
                }
                self.action = Action::None;
            }
            Action::Next(i) => {
                let cursor = self.cursor;
                if cursor + i < self.files.len() {
                    self.cursor = cursor + i;
                } else {
                    self.cursor = self.files.len() - 1;
                }
                self.action = Action::None;
            }
            Action::Back => {
                if let Some(parent) = self.path.parent() {
                    self.path = parent.to_path_buf();
                    self.cursor = 0;
                    self.selected.clear();
                }
                self.action = Action::None;
            }
            Action::Open => {
                let cur_position = &self.files[self.cursor];
                if cur_position.is_dir() {
                    self.path = cur_position.clone();
                    self.cursor = 0;
                    self.selected.clear();
                } else {
                    use std::io::Read;
                    let mut file = std::fs::File::open(cur_position).unwrap();
                    let mut buffer = [0; 1024];
                    let read = file.read(&mut buffer).unwrap();
                    if std::str::from_utf8(&buffer[..read]).is_ok() {
                        self.editor = true;
                    }
                }
                self.action = Action::None;
            }
            Action::Create => {
                self.dialog = Some(Dialog {
                    action: Action::Create,
                    input: "".into(),
                });
                if let Some(ref dialog) = self.dialog {
                    ui::write_backend(dialog, "New file/directory:").unwrap();
                }
                self.action = Action::Pending;
            }
            Action::Delete => {
                self.dialog = Some(Dialog {
                    action: Action::Delete,
                    input: "".into(),
                });
                if let Some(ref dialog) = self.dialog {
                    if self.selected.is_empty() {
                        ui::write_backend(
                            dialog,
                            format!(
                                "Delete \"{}\" ? (y/N)",
                                crate::filename(&self.files[self.cursor])
                            )
                            .as_str(),
                        )
                        .unwrap();
                    } else {
                        let len = self.selected.len();
                        ui::write_backend(dialog, format!("Delete {} items? (y/N)", len).as_str())
                            .unwrap();
                    }
                }
                self.action = Action::Pending;
            }
            Action::Cut => {
                self.is_cut = true;
                self.action = Action::Copy;
            }
            Action::Copy => {
                if self.selected.is_empty() {
                    let file = self.files[self.cursor].clone();
                    ui::log(format!("\"{}\" copied", crate::filename(&file))).unwrap();
                    self.register.push(file);
                } else {
                    ui::log(format!("{} items copied", self.selected.len())).unwrap();
                    self.selected
                        .iter()
                        .for_each(|i| self.register.push(self.files[*i].clone()));
                }
                self.selected.clear();
                shell::clip(&self.register);
                self.action = Action::None;
            }
            Action::Paste => {
                let register = &mut self.register;
                let current_dir = &self.path;
                if self.is_cut {
                    register.iter().for_each(|p| {
                        if let Some(parent) = p.parent() {
                            if &parent.to_path_buf() != current_dir {
                                shell::mv(p, current_dir);
                            }
                        }
                    });
                    ui::log(format!("{} items pasted", register.len())).unwrap();
                    register.clear();
                    self.is_cut = false;
                } else {
                    register.iter().for_each(|p| {
                        if let Some(parent) = p.parent() {
                            if &parent.to_path_buf() != current_dir {
                                shell::cp(p, current_dir);
                            }
                        }
                    });
                    ui::log(format!("{} items pasted", register.len())).unwrap();
                }
                self.action = Action::None;
            }
            Action::Rename => {
                self.dialog = Some(Dialog {
                    action: Action::Rename,
                    input: crate::filename(&self.files[self.cursor]).into(),
                });
                if let Some(ref dialog) = self.dialog {
                    ui::write_backend(
                        dialog,
                        format!("Rename \"{}\" :", crate::filename(&self.files[self.cursor]))
                            .as_str(),
                    )
                    .unwrap();
                }
                self.action = Action::Pending;
            }
            Action::Pending => {}
            Action::PreConfirm => {
                if let Some(dialog) = &self.dialog {
                    if dialog.input.value().is_empty() {
                        self.action = Action::None;
                        self.dialog = None;
                    } else {
                        self.action = Action::Confirm;
                    }
                    let (_, rows) = terminal::size().unwrap();
                    execute!(io::stdout(), MoveTo(0, rows), Clear(ClearType::CurrentLine)).unwrap();
                }
            }
            Action::Confirm => {
                if let Some(Dialog { action, input }) = &self.dialog {
                    let value = input.value();
                    match action {
                        Action::Create => {
                            if let Some(suff) = value.chars().last() {
                                if suff == '/' {
                                    shell::mkdir(&self.path.join(value));
                                } else {
                                    shell::create(&self.path.join(value));
                                }
                                ui::log(format!("\"{}\" created", value)).unwrap();
                            }
                        }
                        Action::Delete => {
                            if value == "y" || value == "Y" {
                                if self.selected.is_empty() {
                                    let file = &self.files[self.cursor];
                                    ui::log(format!("\"{}\" deleted", crate::filename(&file)))
                                        .unwrap();
                                    shell::trash_file(&file);
                                } else {
                                    ui::log(format!("{} items deleted", self.selected.len()))
                                        .unwrap();
                                    self.selected
                                        .iter()
                                        .for_each(|i| shell::trash_file(&self.files[*i]));
                                    self.selected.clear();
                                }
                            }
                        }
                        Action::Rename => {
                            if crate::filename(&self.files[self.cursor]) != value {
                                let file = &self.files[self.cursor];
                                ui::log(format!(
                                    "{} renamed \"{}\"",
                                    crate::filename(&file),
                                    value
                                ))
                                .unwrap();
                                shell::mv(&file, &self.path.join(value));
                            }
                        }
                        _ => {}
                    }
                }
                self.dialog = None;
                self.action = Action::None;
            }
            Action::Clean => {
                let (_, rows) = terminal::size().unwrap();
                execute!(io::stdout(), MoveTo(0, rows), Clear(ClearType::CurrentLine)).unwrap();
                self.dialog = None;
                self.selected.clear();
                self.action = Action::None;
            }
            Action::None => {}
        }
    }

    pub fn handle_dialog(&mut self, event: &Event) {
        if is_pending(&self) {
            if let Some(ref mut dialog) = self.dialog {
                let text = match dialog.action {
                    Action::Create => "New file/directory:".to_string(),
                    Action::Delete => {
                        if self.selected.is_empty() {
                            format!(
                                "Delete \"{}\" ? (y/N)",
                                crate::filename(&self.files[self.cursor])
                            )
                        } else {
                            let len = self.selected.len();
                            format!("Delete {} items? (y/N)", len)
                        }
                    }
                    Action::Rename => {
                        format!("Rename \"{}\" :", crate::filename(&self.files[self.cursor]))
                    }
                    _ => String::new(),
                };
                if dialog.input.handle_event(&event).is_some() {
                    ui::write_backend(&dialog, text.as_str()).unwrap();
                }
            }
        }
    }

    pub fn auto_selector(&mut self) {
        if !self.selected.is_empty() {
            if self.selected[0] <= self.cursor {
                self.selected = (self.selected[0]..=self.cursor).collect();
            } else {
                self.selected = (self.cursor..=self.selected[0]).rev().collect();
            }
        }
    }
}
