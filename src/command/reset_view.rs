use super::Command;

pub struct ResetView;

impl Command for ResetView {
    fn run(&self) -> Result<(), crate::app::Error> {
        crate::cursor::disable_selection();

        Ok(())
    }
}
