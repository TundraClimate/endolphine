use crate::state::State;
use std::{fs, io, path::Path, sync::Arc};

pub fn ask_rename(state: Arc<State>) {
    use crate::misc;

    let files = misc::sorted_child_files(&state.work_dir.get());
    let pos = state.file_view.cursor.current();
    let file = &files[pos];
    let input = &state.input.input;

    super::input_start(&state, &format!("RenameThisItem:{pos}"));

    input.insert(&misc::entry_name(file));

    if let Some(e) = file.extension().and_then(|e| e.to_str()) {
        format!(".{e}").chars().for_each(|_| input.shift_back())
    }
}

pub(super) fn restore_rename(state: Arc<State>) {
    use crate::proc::view;

    view::initialize(&state);
}

pub(super) fn complete_rename(state: &State, content: &str) {
    use crate::misc;

    let wd = state.work_dir.get();
    let child_files = misc::sorted_child_files(&wd);

    if let Some(target) = child_files.get(state.file_view.cursor.current()) {
        let into = wd.join(content);

        if let Err(e) = rename_item(target, &into) {
            crate::log!("Failed to rename item: {}", e.kind());
        }
    }
}

fn rename_item(path: &Path, into: &Path) -> io::Result<()> {
    if !path.exists() || into.exists() {
        return Ok(());
    }

    fs::rename(path, into)
}
