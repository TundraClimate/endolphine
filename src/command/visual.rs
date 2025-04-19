use super::Command;

pub struct VisualSelect;

impl Command for VisualSelect {
    fn run(&self) -> Result<(), crate::Error> {
        if crate::menu::refs().is_enabled() {
            return Ok(());
        }

        crate::cursor::toggle_selection(crate::cursor::load().current());

        Ok(())
    }
}
