use super::Command;
use crate::{app, cursor, input, menu, misc};

pub struct AskRename;

impl Command for AskRename {
    fn run(&self) -> Result<(), crate::Error> {
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

    input::enable(name, Some("Rename".into()));
    crate::sys_log!("i", "Called command: Rename");
    crate::log!("Enter new name for \"{}\"", name);
}

pub struct Rename {
    pub content: String,
}

impl Command for Rename {
    fn run(&self) -> Result<(), crate::Error> {
        let path = app::get_path();

        if let Some(under_cursor_file) =
            misc::sorted_child_files(&path).get(cursor::load().current())
        {
            let renamed = path.join(&self.content);

            let Ok(metadata) = under_cursor_file.symlink_metadata() else {
                crate::sys_log!("w", "Command Rename failed: target metadata cannot access");
                crate::log!("Rename failed: cannot access metadata");

                return Ok(());
            };

            if !under_cursor_file.exists() && !metadata.is_symlink() {
                crate::sys_log!("w", "Command Rename failed: target file not exists");
                crate::log!("Rename failed: \"{}\" is not exists", self.content);

                return Ok(());
            }

            if let Err(e) = std::fs::rename(under_cursor_file, &renamed) {
                crate::sys_log!("w", "Command Rename failed: {}", e.kind());
                crate::log!("Rename failed: {}", e.kind());

                return Ok(());
            }

            crate::sys_log!(
                "i",
                "Command Rename successful: \"{}\" into the \"{}\"",
                under_cursor_file.to_string_lossy(),
                renamed.to_string_lossy()
            );
            crate::log!(
                "\"{}\" renamed to \"{}\"",
                misc::file_name(under_cursor_file),
                misc::file_name(&renamed)
            );
        }

        Ok(())
    }
}
