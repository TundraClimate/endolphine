use super::Command;
use crate::input;

pub struct DisableInput;

impl Command for DisableInput {
    fn run(&self) -> Result<(), crate::app::Error> {
        input::disable();

        Ok(())
    }
}

pub struct InputCursorNext;

impl Command for InputCursorNext {
    fn run(&self) -> Result<(), crate::app::Error> {
        input::cursor_next();

        Ok(())
    }
}

pub struct InputCursorPrev;

impl Command for InputCursorPrev {
    fn run(&self) -> Result<(), crate::app::Error> {
        input::cursor_prev();

        Ok(())
    }
}

pub struct InputInsert(pub char);

impl Command for InputInsert {
    fn run(&self) -> Result<(), crate::app::Error> {
        input::insert(self.0);

        if input::action_is("Search") {
            if let Some(ref buffer) = input::buffer() {
                crate::app::sync_grep(buffer);
            }
        }

        Ok(())
    }
}

pub struct InputDeleteNext;

impl Command for InputDeleteNext {
    fn run(&self) -> Result<(), crate::app::Error> {
        input::delete_cursor_next();

        if input::action_is("Search") {
            if let Some(ref buffer) = input::buffer() {
                crate::app::sync_grep(buffer);
            }
        }

        Ok(())
    }
}

pub struct InputDeleteCurrent;

impl Command for InputDeleteCurrent {
    fn run(&self) -> Result<(), crate::app::Error> {
        input::delete_cursor_pos();

        if input::action_is("Search") {
            if let Some(ref buffer) = input::buffer() {
                crate::app::sync_grep(buffer);
            }
        }

        Ok(())
    }
}

pub struct CompleteInput;

impl Command for CompleteInput {
    fn run(&self) -> Result<(), crate::app::Error> {
        input::complete_input();

        let action = input::take_action();
        let content = input::take_storage();

        tokio::task::spawn_blocking(|| {
            let Some(content) = content else { return };

            if let Some(action) = action {
                crate::app::proc_count_up();
                map_action(content.trim(), action);
                crate::app::proc_count_down();
            }
        });

        Ok(())
    }
}

fn map_action(content: &str, act: String) {
    use crate::command;
    use crate::config;
    if let Err(e) = match act.as_str() {
        "CreateFileOrDir" => command::CreateFileOrDir {
            content: content.to_string(),
            is_file: !content.ends_with("/"),
        }
        .run(),
        "DeleteFileOrDir" if content.eq_ignore_ascii_case("y") => command::DeleteFileOrDir {
            yank_and_native: (config::load().delete.yank, config::load().native_clip),
            use_tmp: config::load().delete.for_tmp,
        }
        .run(),
        "DeleteSelected" if content.eq_ignore_ascii_case("y") => command::DeleteSelected {
            yank_and_native: (config::load().delete.yank, config::load().native_clip),
            use_tmp: config::load().delete.for_tmp,
        }
        .run(),
        "Rename" => command::Rename {
            content: content.to_string(),
        }
        .run(),
        "Paste" => command::Paste {
            overwrite: content.eq_ignore_ascii_case("y"),
            native: config::load().native_clip,
        }
        .run(),
        "Search" => command::SearchNext.run(),
        _ => Ok(()),
    } {
        e.handle()
    }
}
