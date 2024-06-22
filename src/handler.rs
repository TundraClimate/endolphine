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
use std::{
    fs::File,
    io::{self, Read},
};
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
            KeyCode::Esc => self.action = Action::Clean,
            KeyCode::Char('j') if !is_pending(&self) => self.action = Action::Next(1),
            KeyCode::Char('J') if !is_pending(&self) => self.action = Action::Next(10),
            KeyCode::Char('k') if !is_pending(&self) => self.action = Action::Previous(1),
            KeyCode::Char('K') if !is_pending(&self) => self.action = Action::Previous(10),
            KeyCode::Char('v') if !is_pending(&self) => self.selected.push(self.cursor),
            KeyCode::Char('c') if !is_pending(&self) => self.action = Action::Cut,
            KeyCode::Char('y') if !is_pending(&self) => self.action = Action::Copy,
            KeyCode::Char('p') if !is_pending(&self) => self.action = Action::Paste,
            KeyCode::Char('a') if !is_pending(&self) => self.action = Action::Create,
            KeyCode::Char('d') if !is_pending(&self) => self.action = Action::Delete,
            KeyCode::Char('r') if !is_pending(&self) => self.action = Action::Rename,
            KeyCode::Char('h') if !is_pending(&self) => self.action = Action::Back,
            KeyCode::Char('l') if !is_pending(&self) => self.action = Action::Open,
            KeyCode::Enter if is_pending(&self) => self.action = Action::PreConfirm,

            _ => {}
        }
    }

    pub fn handle_action(&mut self) {
        let action = &self.action;
        self.action = match action {
            Action::Previous(i) => {
                let cursor = self.cursor;
                self.cursor = if cursor >= *i { cursor - i } else { 0 };
                Action::None
            }
            Action::Next(i) => {
                let cursor = self.cursor;
                let len = self.files.len();
                self.cursor = if cursor + i < len {
                    cursor + i
                } else {
                    len - 1
                };
                Action::None
            }
            Action::Back => {
                if let Some(parent) = self.path.parent() {
                    self.path = parent.to_path_buf();
                    self.cursor = 0;
                    self.selected.clear();
                }
                Action::None
            }
            Action::Open => {
                let cur_position = &self.files[self.cursor];
                if cur_position.is_dir() {
                    self.path = cur_position.clone();
                    self.cursor = 0;
                    self.selected.clear();
                } else {
                    let mut file = File::open(cur_position).unwrap();
                    let mut buffer = [0; 1024];
                    let read = file.read(&mut buffer).unwrap();
                    if std::str::from_utf8(&buffer[..read]).is_ok() {
                        self.editor = true;
                    }
                }
                Action::None
            }
            Action::Create => {
                let dialog = Dialog::from(Action::Create);
                dialog.write_backend("New file/directory:").unwrap();
                self.dialog = Some(dialog);
                Action::Pending
            }
            Action::Delete => {
                let dialog = Dialog::from(Action::Delete);
                if self.selected.is_empty() {
                    dialog
                        .write_backend(format!(
                            "Delete \"{}\" ? (y/N)",
                            crate::filename(&self.files[self.cursor])
                        ))
                        .unwrap();
                } else {
                    let len = self.selected.len();
                    dialog
                        .write_backend(format!("Delete {} items? (y/N)", len))
                        .unwrap();
                }
                self.dialog = Some(dialog);
                Action::Pending
            }
            Action::Cut => {
                self.is_cut = true;
                Action::Copy
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
                    self.selected.clear();
                }
                shell::clip(&self.register);
                Action::None
            }
            Action::Paste => {
                let register = &mut self.register;
                let current_dir = &self.path;
                let operate = if self.is_cut { shell::mv } else { shell::cp };
                register.iter().for_each(|p| {
                    if let Some(parent) = p.parent() {
                        if &parent != current_dir {
                            operate(p, current_dir);
                        }
                    }
                });

                ui::log(format!("{} items pasted", register.len())).unwrap();

                if self.is_cut {
                    register.clear();
                    self.is_cut = false;
                }
                Action::None
            }
            Action::Rename => {
                let name = crate::filename(&self.files[self.cursor]);
                let dialog = Dialog {
                    action: Action::Rename,
                    input: name.into(),
                };
                dialog
                    .write_backend(format!("Rename \"{}\" :", name))
                    .unwrap();
                self.dialog = Some(dialog);
                Action::Pending
            }
            Action::Pending => Action::Pending,
            Action::PreConfirm => {
                if let Some(dialog) = &self.dialog {
                    let (_, rows) = terminal::size().unwrap();
                    execute!(io::stdout(), MoveTo(0, rows), Clear(ClearType::CurrentLine)).unwrap();
                    if dialog.input.value().is_empty() {
                        self.dialog = None;
                        Action::None
                    } else {
                        Action::Confirm
                    }
                } else {
                    Action::PreConfirm
                }
            }
            Action::Confirm => {
                if let Some(Dialog { action, input }) = &self.dialog {
                    let value = input.value();
                    match action {
                        Action::Create => {
                            if let Some(suff) = value.chars().last() {
                                let operate = if suff == '/' {
                                    shell::mkdir
                                } else {
                                    shell::create
                                };
                                operate(&self.path.join(value));
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
                            let file = &self.files[self.cursor];
                            if crate::filename(&file) != value {
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
                Action::None
            }
            Action::Clean => {
                let (_, rows) = terminal::size().unwrap();
                execute!(io::stdout(), MoveTo(0, rows), Clear(ClearType::CurrentLine)).unwrap();
                self.dialog = None;
                self.selected.clear();
                Action::None
            }
            Action::None => Action::None,
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
                    dialog.write_backend(text).unwrap();
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
