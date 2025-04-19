use super::Command;

pub struct ResetView;

impl Command for ResetView {
    fn run(&self) -> Result<(), crate::Error> {
        crate::cursor::disable_selection();
        crate::canvas::cache_clear();

        Ok(())
    }
}
