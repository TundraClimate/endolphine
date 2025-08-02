mod create;
mod delete;
mod paste;
mod rename;
pub mod search;

use crate::state::State;
use std::sync::Arc;

pub use create::ask_create;
pub use delete::{ask_delete, ask_delete_selects};
pub use paste::ask_paste;
pub use rename::ask_rename;

fn input_start(state: &State, tag: &str) {
    use crate::state::Mode;

    let input = &state.input;

    input.enable(tag);
    input.input.take();

    log::info!("Input start: {tag}");

    state.file_view.selection.disable();
    state.mode.switch(Mode::Input);
}

fn input_start_with_select(state: &State, tag: &str) {
    use crate::state::Mode;

    let input = &state.input;

    input.enable(tag);
    input.input.take();

    log::info!("Input start: {tag}");

    state.mode.switch(Mode::Input);
}

pub fn complete_input(state: Arc<State>) {
    use super::view;

    let input = &state.input;

    let Some(tag) = input.tag() else {
        return;
    };

    let content = { input.input.take() };

    match tag.trim() {
        tag if tag.starts_with("CreateThisItem") => create::complete_create(&state, &content),
        tag if tag.starts_with("DeleteThisItem") => delete::complete_delete(&state, &content),
        tag if tag.starts_with("DeleteItems") => delete::complete_delete_selects(&state, &content),
        tag if tag.starts_with("RenameThisItem") => rename::complete_rename(&state, &content),
        tag if tag.starts_with("PasteFromCb") => paste::complete_paste(&state, &content),
        tag if tag.starts_with("Search") => search::complete_search(&state, &content),

        _ => panic!("Unknown input tag found: {tag}"),
    }

    log::info!("Input end: {tag}");
    view::initialize(&state);
}

pub fn restore(state: Arc<State>) {
    let Some(tag) = state.input.tag() else {
        return;
    };

    let (tag, ctx) = tag.split_once(":").unwrap_or((tag.as_str(), ""));

    match tag {
        "CreateThisItem" => {
            let start_idx = ctx.parse::<usize>().unwrap_or(0);

            create::restore_create(state, start_idx);
        }
        "DeleteThisItem" => delete::restore_delete(state),
        "DeleteItems" => {
            let Some(start_idx) = ctx
                .split_once(";")
                .and_then(|(_, start_idx)| start_idx.parse::<usize>().ok())
            else {
                panic!("Cannot parse the 'DeleteItems' context");
            };

            delete::restore_delete_selects(state, start_idx);
        }
        "RenameThisItem" => rename::restore_rename(state),
        "PasteFromCb" => paste::restore_paste(state),
        "Search" => search::restore_search(state),

        _ => panic!("Unknown input tag found: {tag}"),
    }

    log::info!("Input cancelled: {tag};{ctx}");
}

fn is_logging_tag(tag: &str) -> bool {
    matches!(tag, "DeleteThisItem" | "DeleteItems" | "PasteFromCb")
}

fn logging_input(state: &State) {
    let Some(tag) = state.input.tag() else {
        return;
    };

    let (tag, ctx) = tag.split_once(":").unwrap_or((tag.as_str(), ""));

    if is_logging_tag(tag) {
        let prefix = match tag {
            "DeleteThisItem" => &format!("Delete the '{ctx}' (y/N): "),
            "DeleteItems" => {
                let Some((count, _)) = ctx.split_once(";") else {
                    panic!("Cannot parse the 'DeleteItems' context");
                };

                &format!("Delete {count} items (y/N): ")
            }
            "PasteFromCb" => "Overwrite a file (Y/n): ",

            _ => return,
        };

        crate::log!("{prefix}{}", state.input.input.buf_clone());
    }
}

pub fn put(state: Arc<State>, c: char) {
    state.input.input.put(c);

    logging_input(&state);
}

pub fn pop(state: Arc<State>) {
    state.input.input.pop();

    logging_input(&state);
}

pub fn pop_front(state: Arc<State>) {
    state.input.input.pop_front();

    logging_input(&state);
}

pub fn answer_or_put(state: Arc<State>, c: char) {
    put(state.clone(), c);

    let Some(tag) = state.input.tag() else {
        return;
    };

    let (tag, _) = tag.split_once(":").unwrap_or((tag.as_str(), ""));

    if is_logging_tag(tag) {
        complete_input(state);
    }
}
