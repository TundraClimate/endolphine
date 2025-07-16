use crate::state::State;
use std::sync::Arc;

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
    input_start(&state, "CreateThisItem");
}

pub fn ask_delete(state: Arc<State>) {
    input_start(&state, "DeleteThisItem");
}

pub fn ask_delete_selects(state: Arc<State>) {
    input_start_with_select(&state, "DeleteItems");
}

pub fn ask_paste(state: Arc<State>) {
    input_start(&state, "PasteItems");
}

pub fn ask_rename(state: Arc<State>) {
    input_start(&state, "RenameThisItem");
}
