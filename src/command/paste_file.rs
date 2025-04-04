use super::Command;
use crate::{app, config, input, menu};

pub struct AskPaste;

impl Command for AskPaste {
    fn run(&self) -> Result<(), app::Error> {
        if menu::refs().is_enabled() {
            return Ok(());
        }

        input::use_f_mut(|i| {
            let config = config::load();
            let default_paste_input = if config.paste.default_overwrite {
                "y"
            } else {
                ""
            };

            i.enable(default_paste_input, Some("Paste".into()));

            if config.paste.force_mode {
                crate::handler::handle_input_mode(
                    i,
                    crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Enter),
                );
            } else {
                crate::log!("overwrite the same files? (y/Y)");
            };
        });

        Ok(())
    }
}
