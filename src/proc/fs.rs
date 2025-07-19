use crate::state::State;
use std::{fs, io, path::Path, sync::Arc};

fn input_start(state: &State, tag: &str) {
    use crate::state::Mode;

    let input = &state.input;

    input.enable(tag);
    input.input.take();

    state.file_view.selection.disable();
    state.mode.switch(Mode::Input);
}

fn input_start_with_select(state: &State, tag: &str) {
    use crate::state::Mode;

    let input = &state.input;

    input.enable(tag);
    input.input.take();

    state.mode.switch(Mode::Input);
}

pub fn ask_create(state: Arc<State>) {
    use crate::misc;
    use std::fs;

    let wd = state.work_dir.get();
    let dummy = wd.join(".ep.ed");

    if fs::write(&dummy, b"").is_err() {
        crate::log!("Failed to generate a dummy file");

        return;
    };

    let cursor = &state.file_view.cursor;
    let start_idx = cursor.current();

    cursor.resize(misc::child_files_len(&wd));
    cursor.shift_p(misc::child_files_len(&wd));

    input_start(&state, &format!("CreateThisItem:{start_idx}"));

    crate::log!("Enter name for new File or Directory (for Directory, end with '/')");
}

fn restore_create(state: Arc<State>, start_idx: usize) {
    use super::view;
    use std::fs;

    if fs::remove_file(state.work_dir.get().join(".ep.ed")).is_err() {
        crate::log!("Failed to remove a dummy file");
    }

    view::refresh(state.clone());
    let cursor = &state.file_view.cursor;

    cursor.reset();
    cursor.shift_p(start_idx);
}

fn complete_create(state: &State, content: &str) {
    use crate::misc;

    let is_dir = content.ends_with("/");
    let path = state.work_dir.get().join(content);

    match create_item(&path, is_dir) {
        Ok(_) => {
            if fs::remove_file(state.work_dir.get().join(".ep.ed")).is_err() {
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
        }
        Err(e) => {
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

pub fn ask_delete(state: Arc<State>) {
    use crate::misc;

    let child_files = misc::sorted_child_files(&state.work_dir.get());

    if let Some(item) = child_files.get(state.file_view.cursor.current()) {
        let target_name = misc::entry_name(item);

        input_start(&state, &format!("DeleteThisItem:{target_name}"));
    }
}

fn restore_delete(state: Arc<State>) {
    use super::view;

    view::refresh(state.clone());
}

fn complete_delete(state: Arc<State>, content: &str) {
    use super::view;
    use crate::misc;

    if !content.starts_with("y") {
        view::refresh(state.clone());

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

    input_start_with_select(
        &state,
        &format!("DeleteItems:{};{start_idx}", selection.len()),
    );
}

fn restore_delete_selects(state: Arc<State>, start_idx: usize) {
    use super::view;

    let cursor = &state.file_view.cursor;

    cursor.reset();
    cursor.shift_p(start_idx);

    view::refresh(state.clone());
}

fn complete_delete_selects(state: Arc<State>, content: &str) {
    use super::view;
    use crate::misc;

    if !content.to_ascii_lowercase().starts_with("y") {
        view::refresh(state);

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

pub fn ask_rename(state: Arc<State>) {
    use crate::misc;

    input_start(&state, "RenameThisItem");

    let files = misc::sorted_child_files(&state.work_dir.get());
    let file = &files[state.file_view.cursor.current()];
    let input = &state.input.input;

    input.insert(&misc::entry_name(file));

    if let Some(e) = file.extension().and_then(|e| e.to_str()) {
        format!(".{e}").chars().for_each(|_| input.shift_back())
    }
}

pub fn ask_paste(state: Arc<State>) {
    input_start(&state, "PasteItems");
}

pub fn complete_input(state: Arc<State>) {
    use super::view;

    let input = &state.input;

    let Some(tag) = input.tag() else {
        return;
    };

    let content = { input.input.take() };

    match tag.trim() {
        tag if tag.starts_with("CreateThisItem") => complete_create(&state, &content),
        tag if tag.starts_with("DeleteThisItem") => complete_delete(state.clone(), &content),
        tag if tag.starts_with("DeleteItems") => complete_delete_selects(state.clone(), &content),

        _ => panic!("Unknown input tag found: {tag}"),
    }

    view::refresh(state);
}

pub fn restore(state: Arc<State>) {
    let Some(tag) = state.input.tag() else {
        return;
    };

    let (tag, ctx) = tag.split_once(":").unwrap_or((tag.as_str(), ""));

    match tag {
        "CreateThisItem" => {
            let start_idx = ctx.parse::<usize>().unwrap_or(0);
            restore_create(state, start_idx);
        }
        "DeleteThisItem" => restore_delete(state),
        "DeleteItems" => {
            let Some(start_idx) = ctx
                .split_once(";")
                .and_then(|(_, start_idx)| start_idx.parse::<usize>().ok())
            else {
                panic!("Cannot parse the 'DeleteItems' context");
            };

            restore_delete_selects(state, start_idx);
        }

        _ => panic!("Unknown input tag found: {tag}"),
    }
}
