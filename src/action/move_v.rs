use crate::{action::Action, App};

pub fn previous(app: &mut App, i: usize) -> Action {
    let cursor = app.cursor;
    app.cursor = if cursor >= i { cursor - i } else { 0 };
    Action::None
}

pub fn next(app: &mut App, i: usize) -> Action {
    let cursor = app.cursor;
    let len = app.files.len();
    if len != 0 {
        app.cursor = if cursor + i < len {
            cursor + i
        } else {
            len - 1
        };
    }
    Action::None
}
