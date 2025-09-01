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
        log::info!("Delete cancelled");

        return;
    }

    let child_files = misc::sorted_child_files(&state.work_dir.get());
    let path = child_files.get(state.file_view.cursor.current());

    if let Some(path) = path {
        let name = misc::entry_name(path);

        match delete_item(path) {
            Ok(_) => {
                log::info!("The '{name}' was successfully deleted");
                crate::log!("'{name}' delete successful");
            }
            Err(e) => {
                log::warn!("Delete a '{name}' is failed\n\t{}", e.kind());
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
    use crate::{config, misc, proc::yank};

    log::info!("Delete the {}", misc::entry_name(path));

    if !path.exists() {
        log::warn!(
            "delete {0} failed\n\t{0} is not exists",
            misc::entry_name(path)
        );

        return Ok(());
    }

    let Ok(metadata) = path.symlink_metadata() else {
        log::warn!(
            "delete {} failed\n\tmetadata cannot read",
            misc::entry_name(path)
        );

        return Ok(());
    };

    let config = config::get();

    if config.delete_to_temp {
        log::info!("Move the {} to temp", misc::entry_name(path));

        let trash = Path::new("/tmp/endolphine/Trash");
        let trashed = trash.join(misc::entry_name(path));

        log::info!("Cleaning an item of same name in the trash");

        if trashed.is_symlink() || trashed.is_file() {
            fs::remove_file(&trashed)?;
        } else if trashed.is_dir() {
            fs::remove_dir_all(&trashed)?;
        }

        log::info!("That item successfully cleaned");

        if config.delete_with_yank {
            log::info!("Yank the trashed item");

            yank::clip_files(&[&trashed])?;

            log::info!("The trashed item successfully yanked");
        }

        fs::rename(path, trashed)
    } else if metadata.is_symlink() || metadata.is_file() {
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

    log::info!("Cursor reset to {start_idx}");

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
        log::info!("Delete cancelled");

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
            log::warn!("Delete files is failed\n{}", e.kind());
            crate::log!("Failed to delete items: {}", e.kind());
        }
    }
}

fn delete_items(paths: Vec<&Path>) -> io::Result<()> {
    use crate::{config, proc::yank};

    log::info!(
        "Delete files: \n{:?}",
        paths
            .iter()
            .map(|path| path.to_string_lossy())
            .collect::<Vec<_>>()
    );

    paths.iter().try_for_each(|path| delete_item(path))?;

    let config = config::get();

    if config.delete_to_temp && config.delete_with_yank {
        log::info!("Yank items again");

        yank::clip_files(&paths)?;

        log::info!("The trashed items successfully yanked");
    }

    Ok(())
}
