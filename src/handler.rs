use crate::{
    action::{self, clip, confirm, manage, menu, move_h, move_v, Action},
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
            KeyCode::Char('j') if self.menu_opened() => self.action = Action::Next(1),
            KeyCode::Char('J') if self.menu_opened() => self.action = Action::Next(10),
            KeyCode::Char('k') if self.menu_opened() => self.action = Action::Previous(1),
            KeyCode::Char('K') if self.menu_opened() => self.action = Action::Previous(10),
            KeyCode::Char('V') if !is_pending(&self) => self.selected.push(self.cursor),
            KeyCode::Char('c') if !is_pending(&self) => self.action = Action::Cut,
            KeyCode::Char('y') if !is_pending(&self) => self.action = Action::Copy,
            KeyCode::Char('p') if !is_pending(&self) => self.action = Action::Paste,
            KeyCode::Char('a') if !is_pending(&self) => self.action = Action::Create,
            KeyCode::Char('d') if !is_pending(&self) => self.action = Action::Delete,
            KeyCode::Char('r') if !is_pending(&self) => self.action = Action::Rename,
            KeyCode::Char('h') if !is_pending(&self) => self.action = Action::Back,
            KeyCode::Char('l') if self.menu_opened() => self.action = Action::Select,
            KeyCode::Char('l') if !is_pending(&self) => self.action = Action::Open,
            KeyCode::Char('/') if !is_pending(&self) => self.action = Action::Search,
            KeyCode::Enter if is_pending(&self) => self.action = Action::PreConfirm,
            KeyCode::Enter if !is_pending(&self) => self.action = Action::Menu,

            _ => {}
        }
    }

    pub fn handle_action(&mut self) -> Result<(), Box<dyn Error>> {
        let action = &self.action;
        self.action = match action {
            Action::Previous(i) => move_v::previous(self, *i),
            Action::Next(i) => move_v::next(self, *i),
            Action::Back => move_h::back(self),
            Action::Open => move_h::open(self)?,
            Action::Search => action::search(self)?,
            Action::Create => manage::create(self)?,
            Action::Delete => manage::delete(self)?,
            Action::Cut => clip::cut(self),
            Action::Copy => clip::copy(self)?,
            Action::Paste => clip::paste(self)?,
            Action::Rename => manage::rename(self)?,
            Action::Menu => menu::open(self),
            Action::Select => menu::select(self)?,
            Action::Pending => Action::Pending,
            Action::PreConfirm => confirm::pre_confirm(self)?,
            Action::Confirm => confirm::confirm(self)?,
            Action::Clean => action::clean(self)?,
            Action::None => Action::None,
        };
        Ok(())
    }

    pub fn handle_dialog(&mut self, event: &Event) -> Result<(), Box<dyn Error>> {
        if !is_pending(&self) {
            return Ok(());
        }
        let text = self
            .dialog
            .as_ref()
            .and_then(|dialog| self.dialog_prefix(&dialog.action))
            .unwrap_or_else(String::new);
        if let Some(ref mut dialog) = self.dialog {
            if self.menu.is_none() && dialog.input.handle_event(&event).is_some() {
                dialog.write_backend(text)?;
            }
        }
        Ok(())
    }

    fn dialog_prefix(&self, action: &Action) -> Option<String> {
        match action {
            Action::Create => Some("New file/directory:".to_string()),
            Action::Delete if self.selected.is_empty() => {
                let file = self.files.require(self.cursor)?;
                let filename = crate::filename(file);
                Some(format!("Delete \"{}\" ? (y/N)", filename))
            }
            Action::Delete => Some(format!("Delete {} items? (y/N)", self.selected.len())),
            Action::Rename => {
                let file = self.files.require(self.cursor)?;
                let filename = crate::filename(file);
                Some(format!("Rename \"{}\" :", filename))
            }
            Action::Search => Some("/".into()),
            _ => None,
        }
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
