use super::Command;
use crate::{app, cursor, input, menu};

pub struct Search {
    pub new: bool,
}

impl Command for Search {
    fn run(&self) -> Result<(), app::Error> {
        if menu::refs().is_enabled() {
            return Ok(());
        }

        cursor::disable_selection();

        if self.new {
            app::grep_update(|m| m.clear());
            input::use_f_mut(|i| i.enable("/", Some("Search".to_string())));
        } else if !app::is_regex_empty() {
            input::use_f_mut(|i| {
                i.enable("/", Some("Search".to_string()));
                crate::handler::handle_input_mode(
                    i,
                    crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Enter),
                )
            });
        }

        Ok(())
    }
}
