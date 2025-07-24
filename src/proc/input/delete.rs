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

pub(super) fn complete_delete(state: &State, content: &str) {
    use crate::misc;

    if !content.to_ascii_lowercase().starts_with("y") {
        return;
    }

    let child_files = misc::sorted_child_files(&state.work_dir.get());
    let path = child_files.get(state.file_view.cursor.current());

    if let Some(path) = path
        && let Err(e) = delete_item(path)
    {
        crate::log!(
            "Failed to delete the '{}': {}",
            misc::entry_name(path),
            e.kind()
        );
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

    if let Err(e) = delete_items(paths) {
        crate::log!("Failed to delete items: {}", e.kind());
    }
}

fn delete_items(paths: Vec<&Path>) -> io::Result<()> {
    paths.iter().try_for_each(|path| delete_item(path))
}
