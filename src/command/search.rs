use super::Command;
use crate::{app, cursor, input, menu, misc};

pub struct Search;

impl Command for Search {
    fn run(&self) -> Result<(), app::Error> {
        if menu::refs().is_enabled() {
            return Ok(());
        }

        cursor::disable_selection();
        app::grep_update(|m| m.clear());
        input::use_f_mut(|i| i.enable("/", Some("Search".to_string())));

        Ok(())
    }
}

pub struct SearchNext;

impl Command for SearchNext {
    fn run(&self) -> Result<(), crate::app::Error> {
        let cursor = cursor::load();

        let child_files = misc::sorted_child_files(&app::get_path());
        let first_match_pos = child_files[cursor.current() + 1..]
            .iter()
            .chain(child_files[..cursor.current()].iter())
            .position(|f| app::regex_match(misc::file_name(f)))
            .map(|pos| pos + 1)
            .unwrap_or(0);

        cursor.shift_loop(first_match_pos as isize);
        crate::log!("/{}", app::read_grep());

        Ok(())
    }
}
