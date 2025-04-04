use super::Command;
use crate::{app, cursor, input, menu};

pub struct AskCreate;

impl Command for AskCreate {
    fn run(&self) -> Result<(), app::Error> {
        if menu::refs().is_enabled() {
            return Ok(());
        }

        cursor::disable_selection();
        input::use_f_mut(|i| i.enable("", Some("AddNewFileOrDirectory".into())));
        crate::sys_log!("i", "Called command: AddNewFileOrDirectory");
        crate::log!("Enter name for new File or Directory (for Directory, end with \"/\")");

        Ok(())
    }
}
