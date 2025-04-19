use super::Command;
use crate::{app, cursor, input, menu, misc};

pub struct AskCreate;

impl Command for AskCreate {
    fn run(&self) -> Result<(), crate::Error> {
        if menu::refs().is_enabled() {
            return Ok(());
        }

        cursor::disable_selection();
        input::enable("", Some("CreateFileOrDir".into()));
        crate::sys_log!("i", "Called command: CreateFileOrDir");
        crate::log!("Enter name for new File or Directory (for Directory, end with \"/\")");

        Ok(())
    }
}

pub struct CreateFileOrDir {
    pub content: String,
    pub is_file: bool,
}

impl Command for CreateFileOrDir {
    fn run(&self) -> Result<(), crate::Error> {
        let path = app::get_path().join(&self.content);

        if path.exists() {
            crate::sys_log!(
                "w",
                "Command CreateFileOrDir failed: \"{}\" is already exists",
                self.content
            );
            crate::log!(
                "Add new file failed: \"{}\" is already exists",
                self.content
            );

            return Ok(());
        }

        let add_res = if self.is_file {
            std::fs::write(&path, "")
        } else {
            std::fs::create_dir(&path)
        };

        if let Err(e) = add_res {
            crate::sys_log!("w", "Command CreateFileOrDir failed: {}", e.kind());
            crate::log!("Add new file failed: {}", e.kind());

            return Ok(());
        }

        cursor::load().resize(misc::child_files_len(&app::get_path()));
        crate::sys_log!(
            "w",
            "Command CreateFileOrDir successful: create the {}",
            self.content
        );
        crate::log!("\"{}\" create successful", self.content);

        Ok(())
    }
}
