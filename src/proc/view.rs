use crate::{proc::CommandContext, state::State};
use std::{path::Path, sync::Arc};

pub fn move_cursor(state: Arc<State>, ctx: CommandContext, positive: bool) {
    let cursor = &state.file_view.cursor;
    let point = ctx.prenum.unwrap_or(1);

    if positive {
        cursor.shift_p(point);
    } else {
        cursor.shift_n(point);
    }
}

pub fn move_cursor_too(state: Arc<State>, positive: bool) {
    let cursor = &state.file_view.cursor;
    let point = cursor.len();

    if positive {
        cursor.shift_p(point);
    } else {
        cursor.shift_n(point);
    }
}

pub fn move_page(state: Arc<State>, ctx: CommandContext, positive: bool) {
    let cursor = &state.file_view.cursor;
    let page_len = state.term_size.load().height.saturating_sub(4) as usize;
    let point = page_len * ctx.prenum.unwrap_or(1);

    if positive {
        cursor.shift_p(point);
    } else {
        cursor.shift_n(point);
    }
}

fn move_dir(state: Arc<State>, path: &Path) {
    use crate::{misc, state::Mode};

    state.mode.switch(Mode::Normal);
    state.work_dir.store(path);

    let cursor = &state.file_view.cursor;

    cursor.resize(misc::child_files_len(path));
    cursor.reset();
}

pub fn move_parent(state: Arc<State>) {
    use crate::misc;

    let wd = state.work_dir.get();

    if wd == Path::new("/") {
        return;
    }

    let child_files = misc::sorted_child_files(&wd);
    let cursor = &state.file_view.cursor;
    let recorded_path = child_files.get(cursor.current());
    let parent_path = wd.parent().unwrap_or(&wd);

    move_dir(state.clone(), parent_path);

    if let Some(record) = recorded_path {
        state.file_view.cursor_cache.wrap_node(record);
    }

    let child_files = misc::sorted_child_files(parent_path);

    if let Some(pos) = child_files.into_iter().position(|p| p == wd) {
        cursor.shift_p(pos);
    }
}

pub fn attach_child(state: Arc<State>) {
    use crate::misc;

    let wd = state.work_dir.get();
    let child_files = misc::sorted_child_files(&wd);

    if child_files.is_empty() {
        return;
    }

    let cursor = &state.file_view.cursor;

    let Some(target_path) = child_files.get(cursor.current()) else {
        return;
    };

    if target_path.is_dir() {
        move_dir(state.clone(), target_path);

        let child_files = misc::sorted_child_files(target_path);
        let cursor_cache = &state.file_view.cursor_cache;

        if let Some(pos) = child_files.iter().position(|e| cursor_cache.inner_equal(e)) {
            cursor.shift_p(pos);
            cursor_cache.unwrap_surface();
        } else {
            cursor_cache.reset();
        }
    }
}
