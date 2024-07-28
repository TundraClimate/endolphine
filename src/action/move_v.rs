use crate::{action::Action, App};

pub fn move_cursor(app: &mut App, offset: isize) -> Action {
    let cursor = app.cursor as isize;
    let len = app.finder.len() as isize;

    if len != 0 {
        app.cursor = if offset < 0 {
            (cursor + offset).max(0)
        } else {
            (cursor + offset).min(len - 1)
        } as usize;
    }
    Action::None
}

pub fn previous(app: &mut App, i: usize) -> Action {
    move_cursor(app, -(i as isize))
}

pub fn next(app: &mut App, i: usize) -> Action {
    move_cursor(app, i as isize)
}
