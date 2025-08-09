use crate::state::State;
use std::{fs, io, path::Path, sync::Arc};

pub fn ask_create(state: Arc<State>) {
    use crate::misc;
    use std::fs;

    let wd = state.work_dir.get();
    let dummy = wd.join(".ep.ed");

    log::info!("Create a dummy file");

    if let Err(e) = fs::write(&dummy, b"") {
        log::warn!("Create a dummy file is failed");
        log::warn!("Failed kind: {}", e.kind());
        crate::log!("Failed to generate a dummy file");

        return;
    };

    let cursor = &state.file_view.cursor;
    let start_idx = cursor.current();

    cursor.resize(misc::child_files_len(&wd));
    cursor.shift_p(misc::child_files_len(&wd));

    super::input_start(&state, &format!("CreateThisItem:{start_idx}"));

    crate::log!("Enter name for new File or Directory (for Directory, end with '/')");
}

pub(super) fn restore_create(state: Arc<State>, start_idx: usize) {
    use crate::proc::view;
    use std::fs;

    log::info!("Remove a dummy file");

    if let Err(e) = fs::remove_file(state.work_dir.get().join(".ep.ed")) {
        log::warn!("Remove a dummy file is failed");
        log::warn!("Failed kind: {}", e.kind());
        crate::log!("Failed to remove a dummy file");
    }

    view::refresh(state.clone());
    let cursor = &state.file_view.cursor;

    cursor.reset();
    cursor.shift_p(start_idx);
}

pub(super) fn complete_create(state: &State, content: &str) {
    use crate::misc;

    let is_dir = content.ends_with("/");
    let path = state.work_dir.get().join(content);

    log::info!("Create the '{content}'");

    match create_item(&path, is_dir) {
        Ok(_) => {
            log::info!("Remove a dummy file");

            if let Err(e) = fs::remove_file(state.work_dir.get().join(".ep.ed")) {
                log::warn!("Remove a dummy file is failed");
                log::warn!("Failed kind: {}", e.kind());
                crate::log!("Failed to remove a dummy file");
            }

            let cursor = &state.file_view.cursor;

            cursor.reset();

            let child_files = misc::sorted_child_files(&state.work_dir.get());
            if let Some(pos) = child_files
                .iter()
                .position(|item| misc::entry_name(item) == content)
            {
                cursor.shift_p(pos);
            }

            log::info!("The '{content}' was successfully created");
            crate::log!("'{content}' create successful");
        }
        Err(e) => {
            log::warn!("Create a '{content}' is failed");
            log::warn!("Failed kind: {}", e.kind());
            crate::log!(
                "Failed to create the '{}': {}",
                misc::entry_name(&path),
                e.kind()
            );
        }
    }
}

fn create_item(path: &Path, is_dir: bool) -> io::Result<()> {
    if path.exists() {
        return Ok(());
    }

    if is_dir {
        fs::create_dir(path)
    } else {
        fs::write(path, b"")
    }
}
