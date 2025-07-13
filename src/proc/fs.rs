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

pub fn ask_create(state: Arc<State>) {
    input_start(&state, "CreateFileOrDir");
}
