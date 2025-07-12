use crate::{proc::CommandContext, state::State};
use std::{path::Path, sync::Arc};

pub fn refresh(state: Arc<State>) {
    use crate::state::Mode;

    state.file_view.selection.disable();
    state.mode.switch(Mode::Normal);

    state.canvas_hashes.refresh();
}

fn select_cursor_pos(state: &State) {
    state
        .file_view
        .selection
        .select(state.file_view.cursor.current());
}

pub fn move_cursor(state: Arc<State>, ctx: CommandContext, positive: bool) {
    let cursor = &state.file_view.cursor;
    let point = ctx.prenum.unwrap_or(1);

    if positive {
        cursor.shift_p(point);
    } else {
        cursor.shift_n(point);
    }

    select_cursor_pos(&state);
}

pub fn move_cursor_too(state: Arc<State>, positive: bool) {
    let cursor = &state.file_view.cursor;
    let point = cursor.len();

    if positive {
        cursor.shift_p(point);
    } else {
        cursor.shift_n(point);
    }

    select_cursor_pos(&state);
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

    select_cursor_pos(&state);
}

fn move_dir(state: Arc<State>, path: &Path) {
    use crate::{misc, state::Mode};

    state.mode.switch(Mode::Normal);
    state.work_dir.store(path);

    let cursor = &state.file_view.cursor;

    state.file_view.selection.disable();

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
    use crate::{config, misc, tui};
    use std::process::{Command, Stdio};
    use tokio::task;

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
    } else {
        let config = config::get();

        let info = config
            .hijack
            .get(target_path)
            .unwrap_or(config.hijack.default_ed());
        let hijack_tui = info.hijack;
        let exec = &info.cmd;

        if hijack_tui {
            tui::disable();

            Command::new(&exec.cmd)
                .args(&exec.args)
                .arg(target_path)
                .status()
                .ok();

            tui::enable();

            state.canvas_hashes.refresh();
        } else {
            let target_path = target_path.clone();
            let state = state.clone();

            task::spawn_blocking(move || {
                state.proc_counter.increment();

                Command::new(&exec.cmd)
                    .args(&exec.args)
                    .arg(&target_path)
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status()
                    .ok();

                state.proc_counter.decrement();
            });
        }
    }
}

pub fn toggle_vis(state: Arc<State>) {
    use crate::state::Mode;

    let selection = &state.file_view.selection;

    if selection.is_enable() {
        selection.disable();
        state.mode.switch(Mode::Normal);
    } else {
        selection.enable(state.file_view.cursor.current());
        state.mode.switch(Mode::Visual);
    }
}
