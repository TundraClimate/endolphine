use crate::state::State;
use std::sync::Arc;

pub fn start_search(state: Arc<State>) {
    use crate::state::Mode;

    let input = &state.input;

    input.enable("Search");
    input.input.take();

    state.grep.clear();

    state.file_view.selection.disable();
    state.mode.switch(Mode::Search);

    crate::log!("/");
}

fn log_buffer(state: &State) {
    crate::log!("/{}", state.grep.load());
}

pub fn put(state: Arc<State>, c: char) {
    let previous = state.input.input.buf_clone();

    state.input.input.put(c);
    state.grep.update(state.input.input.buf_clone());

    log::info!("Put a {c} to current search");
    log::info!(
        "Previous: {previous}, Current: {}",
        state.input.input.buf_clone()
    );

    log_buffer(&state);
}

pub fn pop(state: Arc<State>) {
    let previous = state.input.input.buf_clone();

    state.input.input.pop();
    state.grep.update(state.input.input.buf_clone());

    log::info!("Pop from current search");
    log::info!(
        "Previous: {previous}, Current: {}",
        state.input.input.buf_clone()
    );

    log_buffer(&state);
}

pub fn pop_front(state: Arc<State>) {
    let previous = state.input.input.buf_clone();

    state.input.input.pop_front();
    state.grep.update(state.input.input.buf_clone());

    log::info!("Pop front from current input");
    log::info!(
        "Previous: {previous}, Current: {}",
        state.input.input.buf_clone()
    );

    log_buffer(&state);
}

pub fn search_next(state: Arc<State>) {
    complete_search(&state, &state.grep.load());

    log::info!("Move to next matches");
}

pub(super) fn complete_search(state: &State, content: &str) {
    use crate::misc;
    use regex::Regex;

    if let Ok(reg) = Regex::new(content) {
        log::info!("{content} is valid regex");

        let child_files = misc::sorted_child_files(&state.work_dir.get());
        let cursor = &state.file_view.cursor;
        let cursor_pos = cursor.current();

        let first_match = child_files[cursor_pos + 1..]
            .iter()
            .chain(child_files[..cursor_pos].iter())
            .position(|f| reg.is_match(&misc::entry_name(f)))
            .map(|pos| pos + 1)
            .unwrap_or(0);

        if first_match != 0 {
            log::info!(
                "The '{content}' was matched to '{}'",
                child_files[cursor_pos + 1..]
                    .iter()
                    .chain(child_files[..cursor_pos].iter())
                    .find(|f| reg.is_match(&misc::entry_name(f)))
                    .map(|f| f.to_string_lossy().to_string())
                    .unwrap_or("NaN".to_string())
            );
        }

        cursor.shift_loop_p(first_match);
        log_buffer(state);

        log::info!("Cursor move down by {first_match}(loop)");
    }
}

pub(super) fn restore_search(state: Arc<State>) {
    use crate::proc::view;

    view::initialize(&state);
}
