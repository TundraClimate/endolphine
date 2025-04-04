use super::Command;
use crate::{app, cursor, input, menu, misc};

pub struct AskDelete;

impl Command for AskDelete {
    fn run(&self) -> Result<(), app::Error> {
        if menu::refs().is_enabled() {
            return Ok(());
        }

        let cursor = cursor::load();

        if cursor::is_selection() {
            ask_rm_selected();

            return Ok(());
        }

        if let Some(under_cursor_file) =
            misc::sorted_child_files(&app::get_path()).get(cursor.current())
        {
            ask_rm_target(under_cursor_file);
        }

        Ok(())
    }
}

fn ask_rm_selected() {
    let selected_files = misc::sorted_child_files(&app::get_path())
        .into_iter()
        .enumerate()
        .filter_map(|(i, f)| cursor::is_selected(i).then_some(f))
        .collect::<Vec<_>>();

    input::use_f_mut(|i| i.enable("", Some("RmSelected".into())));
    crate::sys_log!("i", "Called command: RmSelected");
    crate::log!("Delete {} items ? (y/Y)", selected_files.len());
}

fn ask_rm_target(under_cursor_file: &std::path::Path) {
    input::use_f_mut(|i| i.enable("", Some("RmFileOrDirectory".into())));
    crate::sys_log!("i", "Called command: RmFileOrDirectory");
    crate::log!("Delete \"{}\" ? (y/Y)", misc::file_name(under_cursor_file));
}
