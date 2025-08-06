use crate::state::State;
use std::{fs, io, path::Path, sync::Arc};

pub fn ask_delete(state: Arc<State>) {
    use crate::misc;

    let child_files = misc::sorted_child_files(&state.work_dir.get());

    if let Some(item) = child_files.get(state.file_view.cursor.current()) {
        let target_name = misc::entry_name(item);

        super::input_start(&state, &format!("DeleteThisItem:{target_name}"));
        crate::log!("Delete the '{target_name}' (y/N): ");
    }
}

pub(super) fn restore_delete(state: Arc<State>) {
    use crate::proc::view;

    view::refresh(state.clone());
}

pub fn delete_just(state: Arc<State>) {
    use crate::proc::view;

    complete_delete(&state, "y");

    view::initialize(&state);
}

pub(super) fn complete_delete(state: &State, content: &str) {
    use crate::misc;

    if !content.to_ascii_lowercase().starts_with("y") {
        return;
    }

    let child_files = misc::sorted_child_files(&state.work_dir.get());
    let path = child_files.get(state.file_view.cursor.current());

    if let Some(path) = path {
        let name = misc::entry_name(path);

        log::info!("Delete a '{name}'");

        match delete_item(path) {
            Ok(_) => {
                log::info!("The '{name}' was successfully deleted");
                crate::log!("'{name}' delete successful");
            }
            Err(e) => {
                log::warn!("Delete a '{name}' is failed");
                log::warn!("Failed kind: {}", e.kind());
                crate::log!(
                    "Failed to delete the '{}': {}",
                    misc::entry_name(path),
                    e.kind()
                );
            }
        }
    }
}

fn delete_item(path: &Path) -> io::Result<()> {
    if !path.exists() {
        return Ok(());
    }

    let Ok(metadata) = path.symlink_metadata() else {
        return Ok(());
    };

    if metadata.is_symlink() || metadata.is_file() {
        fs::remove_file(path)
    } else {
        fs::remove_dir_all(path)
    }
}

pub fn ask_delete_selects(state: Arc<State>) {
    let selection = state.file_view.selection.collect();
    let start_idx = *selection
        .first()
        .unwrap_or(&state.file_view.cursor.current());

    super::input_start_with_select(
        &state,
        &format!("DeleteItems:{};{start_idx}", selection.len()),
    );
    crate::log!("Delete {} items (y/N): ", selection.len());
}

pub(super) fn restore_delete_selects(state: Arc<State>, start_idx: usize) {
    use crate::proc::view;

    let cursor = &state.file_view.cursor;

    cursor.reset();
    cursor.shift_p(start_idx);

    view::refresh(state.clone());
}

pub fn delete_selects_just(state: Arc<State>) {
    use crate::proc::view;

    complete_delete_selects(&state, "y");

    view::initialize(&state);
}

pub(super) fn complete_delete_selects(state: &State, content: &str) {
    use crate::misc;

    if !content.to_ascii_lowercase().starts_with("y") {
        return;
    }

    let child_files = misc::sorted_child_files(&state.work_dir.get());
    let paths = state
        .file_view
        .selection
        .collect()
        .into_iter()
        .filter_map(|idx| child_files.get(idx))
        .map(|path| path.as_path())
        .collect::<Vec<_>>();

    log::info!("Delete files: \n{paths:?}");

    match delete_items(paths.clone()) {
        Ok(_) => {
            log::info!("Files was successfully deleted");
            log::info!("{paths:?}");
            crate::log!("{} items delete successful", paths.len());
        }
        Err(e) => {
            log::warn!("Delete files is failed");
            log::warn!("Failed kind: {}", e.kind());
            crate::log!("Failed to delete items: {}", e.kind());
        }
    }
}

fn delete_items(paths: Vec<&Path>) -> io::Result<()> {
    paths.iter().try_for_each(|path| delete_item(path))
}
