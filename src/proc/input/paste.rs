use crate::state::State;
use std::{io, path::Path, sync::Arc};

pub fn ask_paste(state: Arc<State>) {
    super::input_start(&state, "PasteFromCb");
    crate::log!("Overwrite a file (Y/n): ")
}

pub(super) fn restore_paste(state: Arc<State>) {
    use crate::proc::view;

    view::refresh(state.clone());
}

pub(super) fn complete_paste(state: &State, content: &str) {
    if !content.is_empty() && !content.to_ascii_lowercase().starts_with("y") {
        return;
    }

    if let Err(e) = paste_from_cb(&state.work_dir.get()) {
        crate::log!("Failed to paste from the clipboard: {}", e.kind());
    }
}

fn paste_from_cb(dir: &Path) -> io::Result<()> {
    unimplemented!()
}
