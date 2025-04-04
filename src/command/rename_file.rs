use super::Command;
use crate::{app, cursor, input, menu, misc};

pub struct AskRename;

impl Command for AskRename {
    fn run(&self) -> Result<(), app::Error> {
        if menu::refs().is_enabled() {
            return Ok(());
        }

        let cursor = cursor::load();

        cursor::disable_selection();

        if let Some(under_cursor_file) =
            misc::sorted_child_files(&app::get_path()).get(cursor.current())
        {
            ask_rename(under_cursor_file);
        }

        Ok(())
    }
}

fn ask_rename(under_cursor_file: &std::path::Path) {
    let name = misc::file_name(under_cursor_file);

    input::use_f_mut(|i| i.enable(name, Some("Rename".into())));
    crate::sys_log!("i", "Called command: Rename");
    crate::log!("Enter new name for \"{}\"", name);
}
