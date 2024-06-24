use crate::{
    action::{self, Action},
    app::App,
};
use crossterm::event::{Event, KeyCode, KeyEvent};
use std::error::Error;
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
            KeyCode::Char('V') if !is_pending(&self) => self.selected.push(self.cursor),
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

    pub fn handle_action(&mut self) -> Result<(), Box<dyn Error>> {
        let action = &self.action;
        self.action = match action {
            Action::Previous(i) => action::previous(self, *i),
            Action::Next(i) => action::next(self, *i),
            Action::Back => action::back(self),
            Action::Open => action::open(self)?,
            Action::Create => action::create(self)?,
            Action::Delete => action::delete(self)?,
            Action::Cut => action::cut(self),
            Action::Copy => action::copy(self)?,
            Action::Paste => action::paste(self)?,
            Action::Rename => action::rename(self)?,
            Action::Pending => Action::Pending,
            Action::PreConfirm => action::pre_confirm(self)?,
            Action::Confirm => action::confirm(self)?,
            Action::Clean => action::clean(self)?,
            Action::None => Action::None,
        };
        Ok(())
    }

    pub fn handle_dialog(&mut self, event: &Event) -> Result<(), Box<dyn Error>> {
        if is_pending(&self) {
            if let Some(ref mut dialog) = self.dialog {
                let filename = crate::filename(&self.files[self.cursor]);
                let text = match dialog.action {
                    Action::Create => "New file/directory:".to_string(),
                    Action::Delete if self.selected.is_empty() => {
                        format!("Delete \"{}\" ? (y/N)", filename)
                    }
                    Action::Delete => {
                        format!("Delete {} items? (y/N)", self.selected.len())
                    }
                    Action::Rename => {
                        format!("Rename \"{}\" :", filename)
                    }
                    _ => String::new(),
                };
                if dialog.input.handle_event(&event).is_some() {
                    dialog.write_backend(text)?;
                }
            }
        }
        Ok(())
    }

    pub fn auto_selector(&mut self) {
        if !self.selected.is_empty() {
            let base = self.selected[0];
            let cursor = self.cursor;
            if base <= cursor {
                self.selected = (base..=cursor).collect();
            } else {
                self.selected = (cursor..=base).rev().collect();
            }
        }
    }
}
